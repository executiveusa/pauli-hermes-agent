use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEnvelope {
    pub objective: String,
    pub owner: String,
    pub input: Value,
    pub process: Value,
    pub output: Value,
    pub metric: Value,
    pub feedback_loop: String,
    pub escalation_trigger: String,
    pub approval_gate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantScoped<T> {
    pub id: String,
    pub tenant_id: String,
    pub status: String,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: T,
}
