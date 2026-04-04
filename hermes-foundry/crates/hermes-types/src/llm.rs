use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub id:             String,
    pub name:           String,
    pub path:           String,
    pub size_gb:        Option<f64>,
    pub quantization:   Option<String>,
    pub loaded:         bool,
    pub context_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmServerStatus {
    pub ready:        bool,
    pub loaded_model: Option<String>,
    pub url:          String,
    pub backend:      Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt:      String,
    pub max_tokens:  Option<u32>,
    pub temperature: Option<f32>,
    pub stop:        Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text:           String,
    pub tokens_used:    Option<u32>,
    pub finish_reason:  Option<String>,
}
