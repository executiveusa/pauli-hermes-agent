use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

/// Ports for sidecar services
pub const PORT_HERMES_API:    u16 = 8000;
pub const PORT_LLAMA_SERVER:  u16 = 8080;
pub const PORT_WHISPER_SERVER: u16 = 8090;

/// Global application state — injected into all Tauri commands
#[derive(Default)]
pub struct AppState {
    /// HTTP client (shared, reused for all outbound requests)
    pub http_client: reqwest::Client,

    /// Sidecar process handles (pid tracking)
    pub sidecar_pids: Arc<Mutex<HashMap<String, u32>>>,

    /// Active streaming generation — can be cancelled
    pub active_generation: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,

    /// Current loaded LLM model
    pub loaded_model: Arc<RwLock<Option<String>>>,

    /// Whether whisper server is ready
    pub whisper_ready: Arc<RwLock<bool>>,

    /// Whether llama server is ready
    pub llm_ready: Arc<RwLock<bool>>,

    /// Whether hermes-api is ready
    pub api_ready: Arc<RwLock<bool>>,

    /// Base URLs (can change if ports shift)
    pub hermes_api_url:    Arc<RwLock<String>>,
    pub llama_server_url:  Arc<RwLock<String>>,
    pub whisper_server_url: Arc<RwLock<String>>,
}

impl AppState {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client: client,
            hermes_api_url:    Arc::new(RwLock::new(format!("http://127.0.0.1:{}", PORT_HERMES_API))),
            llama_server_url:  Arc::new(RwLock::new(format!("http://127.0.0.1:{}", PORT_LLAMA_SERVER))),
            whisper_server_url: Arc::new(RwLock::new(format!("http://127.0.0.1:{}", PORT_WHISPER_SERVER))),
            ..Default::default()
        }
    }
}

/// Represents the current status of a single sidecar service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name:    String,
    pub status:  String,   // "running" | "stopped" | "error" | "starting"
    pub port:    Option<u16>,
    pub pid:     Option<u32>,
    pub message: Option<String>,
}
