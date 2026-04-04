/// HermesApiClient — HTTP client for the Python hermes-api backend

use anyhow::Result;
use hermes_types::{ChatRequest, ChatResponse};

pub struct HermesApiClient {
    base_url: String,
    client:   reqwest::Client,
}

impl HermesApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client:   reqwest::Client::new(),
        }
    }

    pub async fn health(&self) -> Result<bool> {
        let resp = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        Ok(resp.status().is_success())
    }

    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let body = serde_json::json!({
            "message":         request.message,
            "conversation_id": request.conversation_id,
            "model":           request.model,
            "stream":          false,
        });

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        let content = json["response"].as_str().unwrap_or("").to_string();

        Ok(ChatResponse {
            content,
            conversation_id: request.conversation_id.clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            model_used: json["model"].as_str().unwrap_or("unknown").to_string(),
            usage: None,
        })
    }

    pub async fn get_runs(&self, limit: u32) -> Result<serde_json::Value> {
        let resp = self.client
            .get(format!("{}/api/runs?limit={}", self.base_url, limit))
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
