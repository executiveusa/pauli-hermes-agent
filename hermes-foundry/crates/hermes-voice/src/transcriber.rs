/// WhisperClient — HTTP client for whisper-server (AtomicBot whisper.cpp fork)
///
/// Communicates with whisper-server's REST API.
/// Compatible with both the upstream whisper.cpp server and AtomicBot's fork.

use anyhow::{Context, Result};
use hermes_types::{TranscriptionResult, TranscriptionSegment, VoiceModel};

pub struct WhisperClient {
    base_url: String,
    client:   reqwest::Client,
}

impl WhisperClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client:   reqwest::Client::new(),
        }
    }

    /// Check if the server is up
    pub async fn is_healthy(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Transcribe an audio file (WAV preferred; 16kHz mono for best results)
    pub async fn transcribe(
        &self,
        audio_path: &str,
        language: Option<&str>,
    ) -> Result<TranscriptionResult> {
        let audio_bytes = tokio::fs::read(audio_path)
            .await
            .context("Failed to read audio file")?;

        let file_name = std::path::Path::new(audio_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();

        let part = reqwest::multipart::Part::bytes(audio_bytes)
            .file_name(file_name)
            .mime_str("audio/wav")?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("response_format", "json");

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let resp = self.client
            .post(format!("{}/inference", self.base_url))
            .multipart(form)
            .send()
            .await
            .context("Whisper request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Whisper error {}: {}", status, body);
        }

        let json: serde_json::Value = resp.json().await?;

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

    /// List available Whisper models in the models directory
    pub fn list_models(models_dir: &std::path::Path) -> Vec<VoiceModel> {
        let model_defs = vec![
            ("tiny.en",        "Whisper Tiny (English)",         78.0f32,  false),
            ("base.en",        "Whisper Base (English)",         148.0,    true),
            ("small.en",       "Whisper Small (English)",        488.0,    false),
            ("medium.en",      "Whisper Medium (English)",       1500.0,   false),
            ("large-v3",       "Whisper Large v3",               3100.0,   false),
            ("large-v3-turbo", "Whisper Large Turbo",            1600.0,   false),
        ];

        model_defs.into_iter().map(|(id, name, size_mb, recommended)| {
            let file = models_dir.join(format!("ggml-{}.bin", id));
            VoiceModel {
                id: id.to_string(),
                name: name.to_string(),
                size_mb,
                downloaded: file.exists(),
                recommended,
            }
        }).collect()
    }
}
