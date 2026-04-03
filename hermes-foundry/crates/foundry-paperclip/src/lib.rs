use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSnapshot {
    pub tenant_id: String,
    pub monthly_cap_usd: f64,
    pub spent_usd: f64,
    pub approvals_required: u32,
}
