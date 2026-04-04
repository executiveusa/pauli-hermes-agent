/// commands/chat.rs — Chat with local LLM or remote Hermes API
///
/// Priority order for model routing:
///   1. Local llama-server (if model loaded and request.use_local == true)
///   2. Remote Hermes API (if api key present)
///   3. Error with helpful message

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message:         String,
    pub conversation_id: Option<String>,
    pub model:           Option<String>,
    pub use_local:       Option<bool>,
    pub system_prompt:   Option<String>,
    pub temperature:     Option<f32>,
    pub max_tokens:      Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatResponse {
    pub content:         String,
    pub conversation_id: String,
    pub model_used:      String,
    pub usage:           Option<TokenUsage>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenUsage {
    pub prompt_tokens:     u32,
    pub completion_tokens: u32,
    pub total_tokens:      u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConversationSummary {
    pub id:           String,
    pub title:        String,
    pub model:        String,
    pub message_count: u32,
    pub created_at:   String,
    pub updated_at:   String,
}

/// Send a message and return a complete (non-streaming) response
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let use_local = request.use_local.unwrap_or(true);
    let llm_ready = *state.llm_ready.read().await;

    if use_local && llm_ready {
        send_via_llama_server(&state, request).await
    } else {
        send_via_hermes_api(&state, request).await
    }
}

/// Send a message and stream tokens back via Tauri events
///
/// Emits `chat:token` events to the frontend as tokens arrive.
/// Emits `chat:done` when complete.
/// Emits `chat:error` on failure.
#[tauri::command]
pub async fn stream_message(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<String, String> {
    let use_local = request.use_local.unwrap_or(true);
    let llm_ready = *state.llm_ready.read().await;
    let conversation_id = request
        .conversation_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let url = if use_local && llm_ready {
        format!("{}/v1/chat/completions", state.llama_server_url.read().await.clone())
    } else {
        format!("{}/api/chat/stream", state.hermes_api_url.read().await.clone())
    };

    let model = request.model.clone()
        .unwrap_or_else(|| state.loaded_model.read().now_or_never()
            .and_then(|g| g.clone())
            .unwrap_or_else(|| "default".to_string()));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": request.system_prompt.unwrap_or_else(|| "You are Hermes, a helpful AI assistant.".to_string())
            },
            {
                "role": "user",
                "content": request.message
            }
        ],
        "stream": true,
        "temperature": request.temperature.unwrap_or(0.7),
        "max_tokens": request.max_tokens.unwrap_or(2048),
    });

    let cid = conversation_id.clone();
    let app_handle = app.clone();
    let client = state.http_client.clone();

    // Spawn streaming in background
    let handle = tauri::async_runtime::spawn(async move {
        match stream_openai_response(&client, &url, body, &cid, &app_handle).await {
            Ok(_) => {}
            Err(e) => {
                let _ = app_handle.emit("chat:error", serde_json::json!({
                    "conversation_id": cid,
                    "error": e.to_string()
                }));
            }
        }
    });

    // Track the handle for cancellation
    *state.active_generation.lock().await = Some(handle);

    Ok(conversation_id)
}

/// Cancel an in-progress streaming generation
#[tauri::command]
pub async fn stop_generation(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.active_generation.lock().await;
    if let Some(handle) = guard.take() {
        handle.abort();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_conversation_history(
    _state: State<'_, AppState>,
    _conversation_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    // In production, fetch from SQLite or Hermes API
    // For now, return empty (frontend stores in Zustand + localStorage)
    Ok(vec![])
}

#[tauri::command]
pub async fn clear_conversation(
    _state: State<'_, AppState>,
    _conversation_id: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn list_conversations(
    _state: State<'_, AppState>,
) -> Result<Vec<ConversationSummary>, String> {
    Ok(vec![])
}

// ── Internal helpers ──────────────────────────────────────────────────────────

async fn send_via_llama_server(
    state: &AppState,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let url = format!("{}/v1/chat/completions", state.llama_server_url.read().await.clone());
    let model = request.model.unwrap_or_else(|| "default".to_string());

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "user", "content": request.message}
        ],
        "stream": false,
        "temperature": request.temperature.unwrap_or(0.7),
        "max_tokens": request.max_tokens.unwrap_or(2048),
    });

    let resp = state.http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("LLM server returned {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(ChatResponse {
        content,
        conversation_id: uuid::Uuid::new_v4().to_string(),
        model_used: model,
        usage: None,
    })
}

async fn send_via_hermes_api(
    state: &AppState,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let url = format!("{}/api/chat", state.hermes_api_url.read().await.clone());
    let body = serde_json::json!({
        "message": request.message,
        "conversation_id": request.conversation_id,
        "model": request.model,
    });

    let resp = state.http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Hermes API request failed: {}", e))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let content = json["response"].as_str().unwrap_or("").to_string();

    Ok(ChatResponse {
        content,
        conversation_id: request.conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        model_used: "hermes-api".to_string(),
        usage: None,
    })
}

/// Stream an OpenAI-compatible SSE response and emit Tauri events for each token
async fn stream_openai_response(
    client: &reqwest::Client,
    url: &str,
    body: serde_json::Value,
    conversation_id: &str,
    app: &AppHandle,
) -> anyhow::Result<()> {
    use futures::StreamExt;

    let resp = client.post(url).json(&body).send().await?;
    let mut stream = resp.bytes_stream();

    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete SSE lines
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    let _ = app.emit("chat:done", serde_json::json!({
                        "conversation_id": conversation_id
                    }));
                    return Ok(());
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(token) = json["choices"][0]["delta"]["content"].as_str() {
                        if !token.is_empty() {
                            let _ = app.emit("chat:token", serde_json::json!({
                                "conversation_id": conversation_id,
                                "token": token
                            }));
                        }
                    }
                }
            }
        }
    }

    let _ = app.emit("chat:done", serde_json::json!({
        "conversation_id": conversation_id
    }));

    Ok(())
}
