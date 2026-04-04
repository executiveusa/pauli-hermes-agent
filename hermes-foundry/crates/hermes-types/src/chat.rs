use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id:              String,
    pub role:            Role,
    pub content:         String,
    pub conversation_id: String,
    pub model:           Option<String>,
    pub created_at:      DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_calls:      Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id:     String,
    pub name:   String,
    pub input:  serde_json::Value,
    pub output: Option<String>,
    pub status: ToolCallStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallStatus {
    Pending,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message:         String,
    pub conversation_id: Option<String>,
    pub model:           Option<String>,
    pub system_prompt:   Option<String>,
    pub temperature:     Option<f32>,
    pub max_tokens:      Option<u32>,
    pub stream:          bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content:         String,
    pub conversation_id: String,
    pub model_used:      String,
    pub usage:           Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens:     u32,
    pub completion_tokens: u32,
    pub total_tokens:      u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id:            String,
    pub title:         String,
    pub model:         String,
    pub message_count: u32,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}
