use thiserror::Error;

#[derive(Debug, Error)]
pub enum HermesError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Service not ready: {service}")]
    ServiceNotReady { service: String },

    #[error("Model not found: {model}")]
    ModelNotFound { model: String },

    #[error("Transcription failed: {0}")]
    Transcription(String),

    #[error("Media processing failed: {0}")]
    Media(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
