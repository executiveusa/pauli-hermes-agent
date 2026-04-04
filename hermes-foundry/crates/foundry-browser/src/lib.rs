use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserRunSpec {
    pub id: String,
    pub tenant_id: String,
    pub allowed_domains: Vec<String>,
    pub status: String,
    pub requires_human_intervention: bool,
}

pub fn should_pause_for_human(signal: &str) -> bool {
    let s = signal.to_lowercase();
    s.contains("captcha") || s.contains("mfa")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captcha_must_pause() {
        assert!(should_pause_for_human("captcha detected"));
    }
}
