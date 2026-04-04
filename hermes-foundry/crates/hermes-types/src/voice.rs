use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text:        String,
    pub language:    Option<String>,
    pub duration_s:  Option<f32>,
    pub segments:    Vec<TranscriptionSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id:    u32,
    pub start: f32,
    pub end:   f32,
    pub text:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceModel {
    pub id:          String,
    pub name:        String,
    pub size_mb:     f32,
    pub downloaded:  bool,
    pub recommended: bool,
}
