use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub id: String,
    pub provider_type: String,
    pub base_url: String,
    pub model: String,
    pub api_key_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatRequest {
    pub provider_profile_id: String,
    pub messages: Vec<Value>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<Value>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatResponse {
    pub provider_profile_id: String,
    pub provider_type: String,
    pub model: String,
    pub response_text: String,
    pub tool_calls: Option<Vec<Value>>,
    pub token_usage_json: Value,
    pub raw_metadata_json: Value,
}

pub fn default_profiles() -> Vec<ProviderProfile> {
    vec![
        ProviderProfile { id: "reasoning_primary".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-5".into(), api_key_env: "OPENAI_API_KEY".into() },
        ProviderProfile { id: "coding_primary".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-4.1".into(), api_key_env: "OPENAI_API_KEY".into() },
        ProviderProfile { id: "research_primary".into(), provider_type: "venice".into(), base_url: std::env::var("VENICE_BASE_URL").unwrap_or_else(|_| "https://api.venice.ai/v1".into()), model: "venice-uncensored".into(), api_key_env: "VENICE_API_KEY".into() },
        ProviderProfile { id: "cheap_summary".into(), provider_type: "gemini".into(), base_url: std::env::var("GEMINI_BASE_URL").unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/openai".into()), model: "gemini-2.5-flash".into(), api_key_env: "GEMINI_API_KEY".into() },
        ProviderProfile { id: "browser_vision".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-4.1".into(), api_key_env: "OPENAI_API_KEY".into() },
        ProviderProfile { id: "fallback_general".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-4.1-mini".into(), api_key_env: "OPENAI_API_KEY".into() },
    ]
}

pub async fn chat(req: LlmChatRequest) -> Result<LlmChatResponse> {
    let profile = default_profiles()
        .into_iter()
        .find(|p| p.id == req.provider_profile_id)
        .ok_or_else(|| anyhow!("unknown provider profile"))?;

    let api_key = std::env::var(&profile.api_key_env).map_err(|_| anyhow!("missing api key env: {}", profile.api_key_env))?;

    let payload = json!({
        "model": profile.model,
        "messages": req.messages,
        "temperature": req.temperature.unwrap_or(0.2),
        "max_tokens": req.max_tokens.unwrap_or(1024),
        "tools": req.tools,
    });

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", profile.base_url.trim_end_matches('/'));
    let resp = client.post(url).bearer_auth(api_key).json(&payload).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let raw: Value = r.json().await.unwrap_or(json!({}));
            let response_text = raw.get("choices").and_then(|c| c.get(0)).and_then(|x| x.get("message")).and_then(|m| m.get("content")).and_then(Value::as_str).unwrap_or("").to_string();
            Ok(LlmChatResponse {
                provider_profile_id: profile.id,
                provider_type: profile.provider_type,
                model: profile.model,
                response_text,
                tool_calls: None,
                token_usage_json: raw.get("usage").cloned().unwrap_or(json!({})),
                raw_metadata_json: raw,
            })
        }
        Ok(r) => Err(anyhow!("provider error: {}", r.status())),
        Err(_) => Ok(LlmChatResponse {
            provider_profile_id: profile.id,
            provider_type: profile.provider_type,
            model: profile.model,
            response_text: "Provider unreachable; returning deterministic fallback summary.".to_string(),
            tool_calls: None,
            token_usage_json: json!({"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}),
            raw_metadata_json: json!({"degraded_mode":true}),
        }),
    }
}
