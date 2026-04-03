use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryScope {
    pub tenant_id: String,
    pub domain: String,
}

pub trait MemoryProvider {
    fn put(&self, _scope: MemoryScope, _text: &str) -> Result<(), String>;
    fn query(&self, _scope: MemoryScope, _query: &str) -> Result<Vec<String>, String>;
}
