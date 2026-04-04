/// commands/voice.rs — Voice input via whisper.cpp sidecar
///
/// The whisper-server exposes:
///   POST /inference    — transcribe an audio file
///   GET  /models       — list available models
///   WS   /audio/stream — real-time streaming transcription

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct TranscriptionResult {
    pub text:       String,
    pub language:   Option<String>,
    pub duration_s: Option<f32>,
    pub segments:   Vec<TranscriptionSegment>,
}

#[derive(Debug, Serialize)]
pub struct TranscriptionSegment {
    pub id:    u32,
    pub start: f32,
    pub end:   f32,
    pub text:  String,
}

#[derive(Debug, Serialize)]
pub struct VoiceModel {
    pub id:          String,
    pub name:        String,
    pub size_mb:     f32,
    pub downloaded:  bool,
    pub recommended: bool,
}

/// Start recording from the system microphone
/// Emits `voice:recording:started` event
/// (Frontend must handle MediaRecorder API for actual audio capture,
///  then call transcribe_file with the recorded blob path)
#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let whisper_ready = *state.whisper_ready.read().await;
    if !whisper_ready {
        return Err("Voice server not ready. Please download a Whisper model first.".to_string());
    }
    app.emit("voice:recording:started", ()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Stop recording (signal to frontend)
#[tauri::command]
pub async fn stop_recording(app: AppHandle) -> Result<(), String> {
    app.emit("voice:recording:stopped", ()).map_err(|e| e.to_string())?;
    Ok(())
}

/// Transcribe an audio file via whisper-server
///
/// Accepts WAV, MP3, WEBM, OGG — auto-converts to 16kHz WAV via FFmpeg if needed
#[tauri::command]
pub async fn transcribe_file(
    state: State<'_, AppState>,
    file_path: String,
    language: Option<String>,
) -> Result<TranscriptionResult, String> {
    let whisper_ready = *state.whisper_ready.read().await;
    if !whisper_ready {
        return Err("Voice server not ready. Download a Whisper model first.".to_string());
    }

    let whisper_url = state.whisper_server_url.read().await.clone();
    let url = format!("{}/inference", whisper_url);

    // Build multipart form
    let file_bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Cannot read audio file: {}", e))?;

    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("audio.wav")
        .to_string();

    let part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(file_name.clone())
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let mut form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("response_format", "json");

    if let Some(lang) = language {
        form = form.text("language", lang);
    }

    let resp = state.http_client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Whisper server error: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Whisper error {}: {}", status, body));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let text = json["text"].as_str().unwrap_or("").trim().to_string();
    let language = json["language"].as_str().map(String::from);

    let segments = json["segments"]
        .as_array()
        .map(|segs| {
            segs.iter().enumerate().map(|(i, seg)| TranscriptionSegment {
                id:    i as u32,
                start: seg["start"].as_f64().unwrap_or(0.0) as f32,
                end:   seg["end"].as_f64().unwrap_or(0.0) as f32,
                text:  seg["text"].as_str().unwrap_or("").trim().to_string(),
            }).collect()
        })
        .unwrap_or_default();

    Ok(TranscriptionResult {
        text,
        language,
        duration_s: json["duration"].as_f64().map(|d| d as f32),
        segments,
    })
}

/// List available Whisper models (downloaded and available to download)
#[tauri::command]
pub async fn list_voice_models(
    state: State<'_, AppState>,
) -> Result<Vec<VoiceModel>, String> {
    let models_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".hermes")
        .join("models");

    let available = vec![
        ("tiny.en",   "Whisper Tiny (English)",  78.0f32,  false),
        ("base.en",   "Whisper Base (English)",  148.0,    true),
        ("small.en",  "Whisper Small (English)", 488.0,    false),
        ("medium.en", "Whisper Medium (English)", 1500.0,  false),
        ("large-v3",  "Whisper Large v3",         3100.0,  false),
        ("large-v3-turbo", "Whisper Large Turbo", 1600.0,  false),
    ];

    let models = available.into_iter().map(|(id, name, size, recommended)| {
        let model_file = models_dir.join(format!("ggml-{}.bin", id));
        VoiceModel {
            id: id.to_string(),
            name: name.to_string(),
            size_mb: size,
            downloaded: model_file.exists(),
            recommended,
        }
    }).collect();

    Ok(models)
}

/// Get whisper-server status
#[tauri::command]
pub async fn get_voice_server_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let ready = *state.whisper_ready.read().await;
    Ok(serde_json::json!({
        "ready": ready,
        "url": state.whisper_server_url.read().await.clone()
    }))
}
