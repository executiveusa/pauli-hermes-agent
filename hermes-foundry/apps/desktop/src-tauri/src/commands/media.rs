/// commands/media.rs — FFmpeg-powered media processing
///
/// Uses FFmpeg sidecar binary for all operations.
/// Supports: audio extraction, transcoding, format conversion, probing.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct MediaInfo {
    pub duration_s:   Option<f64>,
    pub format:       String,
    pub size_bytes:   u64,
    pub video_codec:  Option<String>,
    pub audio_codec:  Option<String>,
    pub width:        Option<u32>,
    pub height:       Option<u32>,
    pub fps:          Option<f64>,
    pub bitrate_kbps: Option<u64>,
    pub sample_rate:  Option<u32>,
    pub channels:     Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ProcessResult {
    pub output_path: String,
    pub duration_ms: u64,
    pub size_bytes:  u64,
}

/// Get metadata about an audio or video file using ffprobe
#[tauri::command]
pub async fn get_media_info(file_path: String) -> Result<MediaInfo, String> {
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            &file_path,
        ])
        .output()
        .await
        .map_err(|e| format!("ffprobe error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| e.to_string())?;

    let format = &json["format"];
    let streams = json["streams"].as_array().cloned().unwrap_or_default();

    let video_stream = streams.iter().find(|s| s["codec_type"] == "video");
    let audio_stream = streams.iter().find(|s| s["codec_type"] == "audio");

    let metadata = std::fs::metadata(&file_path).ok();

    Ok(MediaInfo {
        duration_s: format["duration"].as_str().and_then(|d| d.parse::<f64>().ok()),
        format: format["format_name"].as_str().unwrap_or("unknown").to_string(),
        size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
        video_codec: video_stream.and_then(|v| v["codec_name"].as_str()).map(String::from),
        audio_codec: audio_stream.and_then(|a| a["codec_name"].as_str()).map(String::from),
        width:  video_stream.and_then(|v| v["width"].as_u64()).map(|w| w as u32),
        height: video_stream.and_then(|v| v["height"].as_u64()).map(|h| h as u32),
        fps: video_stream.and_then(|v| v["avg_frame_rate"].as_str()).and_then(|fps| {
            let parts: Vec<&str> = fps.split('/').collect();
            if parts.len() == 2 {
                let num: f64 = parts[0].parse().ok()?;
                let den: f64 = parts[1].parse().ok()?;
                if den > 0.0 { Some(num / den) } else { None }
            } else {
                fps.parse().ok()
            }
        }),
        bitrate_kbps: format["bit_rate"].as_str()
            .and_then(|b| b.parse::<u64>().ok())
            .map(|b| b / 1000),
        sample_rate: audio_stream
            .and_then(|a| a["sample_rate"].as_str())
            .and_then(|s| s.parse().ok()),
        channels: audio_stream
            .and_then(|a| a["channels"].as_u64())
            .map(|c| c as u32),
    })
}

/// Extract audio from a video file
#[tauri::command]
pub async fn extract_audio_from_video(
    input_path: String,
    output_path: Option<String>,
) -> Result<ProcessResult, String> {
    let input = PathBuf::from(&input_path);
    let output = output_path.unwrap_or_else(|| {
        input.with_extension("wav").to_string_lossy().to_string()
    });

    let start = std::time::Instant::now();
    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", &input_path,
            "-vn",
            "-acodec", "pcm_s16le",
            "-ar", "44100",
            "-ac", "2",
            &output,
        ])
        .output()
        .await
        .map_err(|e| format!("FFmpeg error: {}", e))?;

    if !result.status.success() {
        return Err(format!(
            "FFmpeg audio extraction failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    let size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
    Ok(ProcessResult {
        output_path: output,
        duration_ms: start.elapsed().as_millis() as u64,
        size_bytes: size,
    })
}

/// Convert audio to 16kHz mono WAV (required by whisper.cpp)
#[tauri::command]
pub async fn convert_audio_for_whisper(
    input_path: String,
    output_path: Option<String>,
) -> Result<ProcessResult, String> {
    let input = PathBuf::from(&input_path);
    let output = output_path.unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default().to_string_lossy();
        input.with_file_name(format!("{}_whisper.wav", stem)).to_string_lossy().to_string()
    });

    let start = std::time::Instant::now();
    let result = tokio::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i", &input_path,
            "-ar", "16000",     // 16kHz sample rate
            "-ac", "1",         // mono
            "-c:a", "pcm_s16le", // 16-bit PCM
            &output,
        ])
        .output()
        .await
        .map_err(|e| format!("FFmpeg error: {}", e))?;

    if !result.status.success() {
        return Err(format!(
            "FFmpeg conversion failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    let size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
    Ok(ProcessResult {
        output_path: output,
        duration_ms: start.elapsed().as_millis() as u64,
        size_bytes: size,
    })
}

/// Process audio (transcode, normalize, trim)
#[tauri::command]
pub async fn process_audio(
    input_path: String,
    output_path: String,
    format: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u32>,
) -> Result<ProcessResult, String> {
    let start = std::time::Instant::now();
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(), input_path,
    ];

    if let Some(sr) = sample_rate {
        args.extend(["-ar".to_string(), sr.to_string()]);
    }
    if let Some(ch) = channels {
        args.extend(["-ac".to_string(), ch.to_string()]);
    }

    args.push(output_path.clone());

    let result = tokio::process::Command::new("ffmpeg")
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("FFmpeg error: {}", e))?;

    if !result.status.success() {
        return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&result.stderr)));
    }

    let size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
    Ok(ProcessResult {
        output_path,
        duration_ms: start.elapsed().as_millis() as u64,
        size_bytes: size,
    })
}

/// Process video (transcode, resize, compress)
#[tauri::command]
pub async fn process_video(
    input_path: String,
    output_path: String,
    width: Option<u32>,
    height: Option<u32>,
    crf: Option<u8>,
) -> Result<ProcessResult, String> {
    let start = std::time::Instant::now();
    let mut args = vec!["-y".to_string(), "-i".to_string(), input_path];

    // Scale filter
    if width.is_some() || height.is_some() {
        let w = width.map(|w| w.to_string()).unwrap_or_else(|| "-2".to_string());
        let h = height.map(|h| h.to_string()).unwrap_or_else(|| "-2".to_string());
        args.extend(["-vf".to_string(), format!("scale={}:{}", w, h)]);
    }

    // Quality (CRF for H.264)
    args.extend([
        "-c:v".to_string(), "libx264".to_string(),
        "-crf".to_string(), crf.unwrap_or(23).to_string(),
        "-preset".to_string(), "fast".to_string(),
        "-c:a".to_string(), "aac".to_string(),
    ]);

    args.push(output_path.clone());

    let result = tokio::process::Command::new("ffmpeg")
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("FFmpeg error: {}", e))?;

    if !result.status.success() {
        return Err(format!("FFmpeg video processing failed: {}", String::from_utf8_lossy(&result.stderr)));
    }

    let size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
    Ok(ProcessResult {
        output_path,
        duration_ms: start.elapsed().as_millis() as u64,
        size_bytes: size,
    })
}
