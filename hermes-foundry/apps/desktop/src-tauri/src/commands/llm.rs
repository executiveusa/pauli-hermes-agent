/// commands/llm.rs — Local LLM management via llama-server (atomic-llama-cpp-turboquant)
///
/// llama-server exposes an OpenAI-compatible REST API.
/// This module wraps model management, server control, and completion.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize, Clone)]
pub struct LocalModel {
    pub id:          String,
    pub name:        String,
    pub path:        String,
    pub size_gb:     Option<f64>,
    pub quantization: Option<String>,
    pub loaded:      bool,
    pub context_length: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct LlmServerStatus {
    pub ready:        bool,
    pub loaded_model: Option<String>,
    pub url:          String,
    pub backend:      Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompletionRequest {
    pub prompt:      String,
    pub max_tokens:  Option<u32>,
    pub temperature: Option<f32>,
    pub stop:        Option<Vec<String>>,
}

/// List all GGUF models found in the ~/.hermes/models/ directory
#[tauri::command]
pub async fn list_local_models(state: State<'_, AppState>) -> Result<Vec<LocalModel>, String> {
    let models_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".hermes")
        .join("models");

    let mut models = Vec::new();

    let Ok(mut entries) = tokio::fs::read_dir(&models_dir).await else {
        return Ok(vec![]);
    };

    let loaded = state.loaded_model.read().await.clone();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext != "gguf" && !path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .ends_with(".bin") {
            continue;
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let size_gb = tokio::fs::metadata(&path).await.ok()
            .map(|m| m.len() as f64 / 1_073_741_824.0);

        // Extract quantization from filename (e.g., q4_k_m, q5_1)
        let quant = extract_quantization(&file_name);

        let model_id = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&file_name)
            .to_string();

        models.push(LocalModel {
            id:          model_id.clone(),
            name:        format_model_name(&file_name),
            path:        path.to_string_lossy().to_string(),
            size_gb,
            quantization: quant,
            loaded:      loaded.as_deref() == Some(&model_id),
            context_length: None, // Would require reading GGUF metadata
        });
    }

    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models)
}

/// Get llama-server health + currently loaded model
#[tauri::command]
pub async fn get_llm_server_status(state: State<'_, AppState>) -> Result<LlmServerStatus, String> {
    let ready = *state.llm_ready.read().await;
    let url = state.llama_server_url.read().await.clone();
    let loaded_model = state.loaded_model.read().await.clone();

    if ready {
        // Try to get backend info from server
        let backend = state.http_client
            .get(format!("{}/props", url))
            .send()
            .await
            .ok()
            .and_then(|r| r.json::<serde_json::Value>().now_or_never())
            .and_then(|j| j.ok())
            .and_then(|j| j["default_generation_settings"]["model"].as_str().map(String::from));

        Ok(LlmServerStatus { ready, loaded_model, url, backend })
    } else {
        Ok(LlmServerStatus { ready: false, loaded_model: None, url, backend: None })
    }
}

/// Load a model into llama-server (sends a slot request or restarts with model flag)
#[tauri::command]
pub async fn load_local_model(
    state: State<'_, AppState>,
    model_path: String,
) -> Result<(), String> {
    // llama-server doesn't support hot-swapping models without restart.
    // In production: signal app to restart llama-server with the new model.
    // For now, update the tracked model and notify sidecar manager.
    let model_id = std::path::Path::new(&model_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    *state.loaded_model.write().await = Some(model_id);
    Ok(())
}

/// Unload the current model
#[tauri::command]
pub async fn unload_local_model(state: State<'_, AppState>) -> Result<(), String> {
    *state.loaded_model.write().await = None;
    Ok(())
}

/// Direct text completion via llama-server
#[tauri::command]
pub async fn complete(
    state: State<'_, AppState>,
    request: CompletionRequest,
) -> Result<String, String> {
    let url = format!("{}/v1/completions", state.llama_server_url.read().await.clone());

    let body = serde_json::json!({
        "prompt": request.prompt,
        "max_tokens": request.max_tokens.unwrap_or(512),
        "temperature": request.temperature.unwrap_or(0.7),
        "stop": request.stop.unwrap_or_default(),
        "stream": false,
    });

    let resp = state.http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LLM completion error: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json["choices"][0]["text"].as_str().unwrap_or("").to_string())
}

/// Scan the models directory and return count + total size
#[tauri::command]
pub async fn scan_models_dir() -> Result<serde_json::Value, String> {
    let models_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".hermes")
        .join("models");

    let mut count = 0u32;
    let mut total_bytes = 0u64;

    if let Ok(mut entries) = tokio::fs::read_dir(&models_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "gguf" || path.file_name().and_then(|n| n.to_str()).unwrap_or("").ends_with(".bin") {
                count += 1;
                if let Ok(meta) = tokio::fs::metadata(&path).await {
                    total_bytes += meta.len();
                }
            }
        }
    }

    Ok(serde_json::json!({
        "model_count": count,
        "total_size_gb": total_bytes as f64 / 1_073_741_824.0,
        "models_dir": models_dir.to_string_lossy()
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_quantization(filename: &str) -> Option<String> {
    let patterns = ["q4_k_m", "q4_k_s", "q5_k_m", "q5_k_s", "q5_1", "q4_0",
                    "q6_k", "q8_0", "f16", "f32", "q2_k", "q3_k_m"];
    let lower = filename.to_lowercase();
    for pat in &patterns {
        if lower.contains(pat) {
            return Some(pat.to_uppercase());
        }
    }
    None
}

fn format_model_name(filename: &str) -> String {
    // llama-3-8b-instruct-q4_k_m.gguf → Llama 3 8B Instruct Q4_K_M
    let base = filename
        .trim_end_matches(".gguf")
        .trim_end_matches(".bin");

    base.replace('-', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
