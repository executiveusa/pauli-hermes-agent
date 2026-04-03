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
    pub task_types: Vec<String>,
    pub max_tokens_default: u32,
    pub max_spend_per_run_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider_profile_id: String,
    pub healthy: bool,
    pub latency_ms: u64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmChatRequest {
    pub provider_profile_id: String,
    pub task_type: Option<String>,
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
        ProviderProfile { id: "reasoning_primary".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-5".into(), api_key_env: "OPENAI_API_KEY".into(), task_types: vec!["reasoning".into(), "planning".into()], max_tokens_default: 2048, max_spend_per_run_usd: 12.0 },
        ProviderProfile { id: "coding_primary".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-4.1".into(), api_key_env: "OPENAI_API_KEY".into(), task_types: vec!["coding".into()], max_tokens_default: 4096, max_spend_per_run_usd: 20.0 },
        ProviderProfile { id: "research_primary".into(), provider_type: "venice".into(), base_url: std::env::var("VENICE_BASE_URL").unwrap_or_else(|_| "https://api.venice.ai/v1".into()), model: "venice-uncensored".into(), api_key_env: "VENICE_API_KEY".into(), task_types: vec!["research".into(), "proposal".into()], max_tokens_default: 4096, max_spend_per_run_usd: 15.0 },
        ProviderProfile { id: "cheap_summary".into(), provider_type: "gemini".into(), base_url: std::env::var("GEMINI_BASE_URL").unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/openai".into()), model: "gemini-2.5-flash".into(), api_key_env: "GEMINI_API_KEY".into(), task_types: vec!["summary".into(), "bulk".into()], max_tokens_default: 1024, max_spend_per_run_usd: 3.0 },
        ProviderProfile { id: "browser_vision".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-4.1".into(), api_key_env: "OPENAI_API_KEY".into(), task_types: vec!["browser_vision".into()], max_tokens_default: 1536, max_spend_per_run_usd: 8.0 },
        ProviderProfile { id: "fallback_general".into(), provider_type: "openai-compatible".into(), base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into()), model: "gpt-4.1-mini".into(), api_key_env: "OPENAI_API_KEY".into(), task_types: vec!["*".into()], max_tokens_default: 1024, max_spend_per_run_usd: 5.0 },
    ]
}

pub fn provider_health_snapshot() -> Vec<ProviderHealth> {
    default_profiles()
        .into_iter()
        .map(|p| ProviderHealth {
            provider_profile_id: p.id,
            healthy: true,
            latency_ms: 120,
            error_rate: 0.01,
        })
        .collect()
}

pub async fn chat(req: LlmChatRequest) -> Result<LlmChatResponse> {
    let profiles = default_profiles();
    let mut profile = profiles
        .iter()
        .find(|p| p.id == req.provider_profile_id)
        .cloned()
        .ok_or_else(|| anyhow!("unknown provider profile"))?;

    if let Some(task) = &req.task_type {
        let allowed = profile.task_types.iter().any(|t| t == task || t == "*");
        if !allowed {
            profile = profiles
                .iter()
                .find(|p| p.id == "fallback_general")
                .cloned()
                .ok_or_else(|| anyhow!("missing fallback profile"))?;
        }
    }

    let api_key = std::env::var(&profile.api_key_env)
        .map_err(|_| anyhow!("missing api key env: {}", profile.api_key_env))?;

    let payload = json!({
        "model": profile.model,
        "messages": req.messages,
        "temperature": req.temperature.unwrap_or(0.2),
        "max_tokens": req.max_tokens.unwrap_or(profile.max_tokens_default),
        "tools": req.tools,
    });

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", profile.base_url.trim_end_matches('/'));
    let resp = client.post(url).bearer_auth(api_key).json(&payload).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let raw: Value = r.json().await.unwrap_or(json!({}));
            let response_text = raw
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|x| x.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
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
            response_text: "Provider unavailable. Generated fallback operator-safe summary.".to_string(),
            tool_calls: None,
            token_usage_json: json!({"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}),
            raw_metadata_json: json!({"degraded_mode":true}),
        }),
    }
}
