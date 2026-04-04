/// LlamaClient — OpenAI-compatible HTTP client for llama-server
///
/// Works with atomic-llama-cpp-turboquant's llama-server which exposes:
///   GET  /v1/models                — list loaded models
///   POST /v1/chat/completions      — chat (streaming or not)
///   POST /v1/completions           — text completion
///   GET  /health                   — server health
///   GET  /props                    — server properties

use anyhow::Result;
use futures::StreamExt;
use hermes_types::{CompletionRequest, CompletionResponse, LocalModel};

pub struct LlamaClient {
    base_url: String,
    client:   reqwest::Client,
}

impl LlamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client:   reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn is_healthy(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Get the currently loaded model from llama-server
    pub async fn get_loaded_model(&self) -> Option<String> {
        let resp = self.client
            .get(format!("{}/v1/models", self.base_url))
            .send()
            .await
            .ok()?;
        let json: serde_json::Value = resp.json().await.ok()?;
        json["data"][0]["id"].as_str().map(String::from)
    }

    /// Non-streaming text completion
    pub async fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let body = serde_json::json!({
            "prompt": request.prompt,
            "max_tokens": request.max_tokens.unwrap_or(512),
            "temperature": request.temperature.unwrap_or(0.7),
            "stop": request.stop.clone().unwrap_or_default(),
            "stream": false,
        });

        let resp = self.client
            .post(format!("{}/v1/completions", self.base_url))
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        let text = json["choices"][0]["text"].as_str().unwrap_or("").to_string();
        let tokens = json["usage"]["completion_tokens"].as_u64().map(|t| t as u32);
        let finish_reason = json["choices"][0]["finish_reason"].as_str().map(String::from);

        Ok(CompletionResponse {
            text,
            tokens_used:   tokens,
            finish_reason,
        })
    }

    /// Streaming chat completion — yields tokens via a channel
    pub async fn stream_chat(
        &self,
        messages: Vec<serde_json::Value>,
        model: &str,
        temperature: f32,
        max_tokens: u32,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<()> {
        let body = serde_json::json!({
            "model":       model,
            "messages":    messages,
            "stream":      true,
            "temperature": temperature,
            "max_tokens":  max_tokens,
        });

        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body)
            .send()
            .await?;

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { return Ok(()); }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(token) = json["choices"][0]["delta"]["content"].as_str() {
                            if !token.is_empty() {
                                let _ = tx.send(token.to_string()).await;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
