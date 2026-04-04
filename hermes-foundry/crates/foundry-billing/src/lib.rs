use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub tenant_id: String,
    pub run_id: String,
    pub task_type: String,
    pub provider: String,
    pub model: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub provider_cost_estimate_usd: f64,
    pub browser_runtime_minutes: f64,
    pub coding_runtime_minutes: f64,
    pub storage_gb: f64,
    pub workflows_run: u64,
    pub active_subagents: u64,
    pub api_calls: u64,
    pub mcp_calls: u64,
    pub billable_event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingRule {
    pub plan: String,
    pub token_in_per_million_usd: f64,
    pub token_out_per_million_usd: f64,
    pub browser_minute_usd: f64,
    pub coding_minute_usd: f64,
    pub storage_gb_usd: f64,
    pub api_call_usd: f64,
    pub mcp_call_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsageSnapshot {
    pub tenant_id: String,
    pub month: String,
    pub total_estimated_cost_usd: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub total_browser_minutes: f64,
    pub total_coding_minutes: f64,
    pub total_api_calls: u64,
    pub total_mcp_calls: u64,
    pub budget_warning: bool,
}

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("budget exceeded for tenant {tenant_id}: {spent_usd:.4} > {cap_usd:.4}")]
    Exceeded {
        tenant_id: String,
        spent_usd: f64,
        cap_usd: f64,
    },
}

pub fn estimate_cost(rule: &PricingRule, event: &UsageEvent) -> f64 {
    (event.tokens_in as f64 / 1_000_000.0) * rule.token_in_per_million_usd
        + (event.tokens_out as f64 / 1_000_000.0) * rule.token_out_per_million_usd
        + event.browser_runtime_minutes * rule.browser_minute_usd
        + event.coding_runtime_minutes * rule.coding_minute_usd
        + event.storage_gb * rule.storage_gb_usd
        + event.api_calls as f64 * rule.api_call_usd
        + event.mcp_calls as f64 * rule.mcp_call_usd
}

pub fn enforce_budget(tenant_id: &str, spent_usd: f64, cap_usd: f64) -> Result<(), BudgetError> {
    if spent_usd > cap_usd {
        Err(BudgetError::Exceeded {
            tenant_id: tenant_id.to_string(),
            spent_usd,
            cap_usd,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_is_positive_for_usage() {
        let rule = PricingRule {
            plan: "pay_as_you_go".into(),
            token_in_per_million_usd: 2.0,
            token_out_per_million_usd: 6.0,
            browser_minute_usd: 0.04,
            coding_minute_usd: 0.08,
            storage_gb_usd: 0.1,
            api_call_usd: 0.001,
            mcp_call_usd: 0.001,
        };
        let event = UsageEvent {
            tenant_id: "t".into(),
            run_id: "r".into(),
            task_type: "task".into(),
            provider: "p".into(),
            model: "m".into(),
            tokens_in: 1000,
            tokens_out: 2000,
            provider_cost_estimate_usd: 0.0,
            browser_runtime_minutes: 1.0,
            coding_runtime_minutes: 1.0,
            storage_gb: 0.1,
            workflows_run: 1,
            active_subagents: 0,
            api_calls: 1,
            mcp_calls: 1,
            billable_event_type: "x".into(),
        };
        assert!(estimate_cost(&rule, &event) > 0.0);
    }
}
