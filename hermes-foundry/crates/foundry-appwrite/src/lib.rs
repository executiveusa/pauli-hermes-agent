use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppwriteBinding {
    pub tenant_id: String,
    pub project_id: String,
    pub endpoint: String,
}
