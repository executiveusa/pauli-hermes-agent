use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingSessionSpec {
    pub id: String,
    pub tenant_id: String,
    pub repo_url: String,
    pub status: String,
    pub checkpoint: String,
}
