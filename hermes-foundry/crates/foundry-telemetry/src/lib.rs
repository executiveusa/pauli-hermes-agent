use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorError {
    pub code: String,
    pub summary: String,
    pub detail: String,
}
