use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub tenant_id: String,
    pub run_id: String,
    pub task_type: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: u64,
    pub compute_ms: u64,
    pub browser_ms: u64,
    pub worker_ms: u64,
    pub estimated_cost_usd: f64,
    pub billable_event_type: String,
}

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("budget exceeded for tenant {tenant_id}: {spent_usd:.4} > {cap_usd:.4}")]
    Exceeded { tenant_id: String, spent_usd: f64, cap_usd: f64 },
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
