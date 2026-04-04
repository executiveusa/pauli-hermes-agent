/// AgentEngine — Routes chat completions to the appropriate backend
///
/// Priority: local llama-server → remote Hermes API

use anyhow::Result;
use futures::Stream;
use std::pin::Pin;

use hermes_types::{ChatRequest, ChatResponse, HermesError};

use crate::client::HermesApiClient;

pub struct AgentEngine {
    pub hermes_client: HermesApiClient,
    pub llama_url:     String,
    pub http:          reqwest::Client,
}

impl AgentEngine {
    pub fn new(hermes_url: &str, llama_url: &str) -> Self {
        Self {
            hermes_client: HermesApiClient::new(hermes_url),
            llama_url:     llama_url.to_string(),
            http:           reqwest::Client::new(),
        }
    }

    /// Non-streaming: send a message and get a complete response
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        if request.model.as_deref() == Some("local") {
            self.chat_local(&request).await
        } else {
            self.hermes_client.chat(&request).await
        }
    }

    async fn chat_local(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let model = request.model.clone().unwrap_or_else(|| "default".to_string());
        let body = serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": request.message}],
            "stream": false,
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens.unwrap_or(2048),
        });

        let resp = self.http
            .post(format!("{}/v1/chat/completions", self.llama_url))
            .json(&body)
            .send()
            .await
            .map_err(HermesError::Http)?;

        let json: serde_json::Value = resp.json().await.map_err(HermesError::Http)?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            content,
            conversation_id: request.conversation_id.clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            model_used: model,
            usage: None,
        })
    }
}
