use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub duration_s:    Option<f64>,
    pub format:        String,
    pub size_bytes:    u64,
    pub video_codec:   Option<String>,
    pub audio_codec:   Option<String>,
    pub width:         Option<u32>,
    pub height:        Option<u32>,
    pub fps:           Option<f64>,
    pub bitrate_kbps:  Option<u64>,
    pub sample_rate:   Option<u32>,
    pub channels:      Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub output_path:  String,
    pub duration_ms:  u64,
    pub size_bytes:   u64,
}
