use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentSpec {
    pub id: String,
    pub tenant_id: String,
    pub mission: String,
    pub scope: Vec<String>,
    pub status: String,
}
