use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingSecret {
    pub key: String,
    pub reason: String,
    pub unlocks: String,
    pub optional: bool,
}

pub fn inspect_missing_secrets() -> Vec<MissingSecret> {
    let checks = vec![
        ("OPENAI_API_KEY", "LLM routing", "reasoning/coding model calls", false),
        ("GEMINI_API_KEY", "cheap summary lane", "lower-cost summarization", true),
        ("VENICE_API_KEY", "research lane", "privacy-first long-form reasoning", true),
        ("PAPERCLIP_API_KEY", "governance", "budget/approval sync", false),
        ("APPWRITE_API_KEY", "product backend", "project provisioning", true),
        ("COMPOSIO_API_KEY", "SaaS integrations", "external automations", true),
        ("TELEGRAM_BOT_TOKEN", "ops channel", "approval/status messaging", true),
    ];

    checks
        .into_iter()
        .filter(|(k, _, _, _)| std::env::var(k).ok().filter(|v| !v.is_empty()).is_none())
        .map(|(k, reason, unlocks, optional)| MissingSecret {
            key: k.to_string(),
            reason: reason.to_string(),
            unlocks: unlocks.to_string(),
            optional,
        })
        .collect()
}
