/// MediaProcessor — FFmpeg subprocess wrapper

use anyhow::{Context, Result};
use hermes_types::{MediaInfo, ProcessResult};
use std::path::Path;
use std::time::Instant;

pub struct MediaProcessor {
    ffmpeg_path:  String,
    ffprobe_path: String,
}

impl MediaProcessor {
    pub fn new() -> Self {
        Self {
            ffmpeg_path:  std::env::var("FFMPEG_PATH").unwrap_or_else(|_| "ffmpeg".to_string()),
            ffprobe_path: std::env::var("FFPROBE_PATH").unwrap_or_else(|_| "ffprobe".to_string()),
        }
    }

    /// Probe file metadata
    pub async fn probe(&self, input: &str) -> Result<MediaInfo> {
        let output = tokio::process::Command::new(&self.ffprobe_path)
            .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", input])
            .output()
            .await
            .context("ffprobe not found")?;

        if !output.status.success() {
            anyhow::bail!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let format  = &json["format"];
        let streams = json["streams"].as_array().cloned().unwrap_or_default();

        let video = streams.iter().find(|s| s["codec_type"] == "video");
        let audio = streams.iter().find(|s| s["codec_type"] == "audio");
        let size  = std::fs::metadata(input).map(|m| m.len()).unwrap_or(0);

        Ok(MediaInfo {
            duration_s:   format["duration"].as_str().and_then(|d| d.parse().ok()),
            format:       format["format_name"].as_str().unwrap_or("unknown").to_string(),
            size_bytes:   size,
            video_codec:  video.and_then(|v| v["codec_name"].as_str()).map(String::from),
            audio_codec:  audio.and_then(|a| a["codec_name"].as_str()).map(String::from),
            width:        video.and_then(|v| v["width"].as_u64()).map(|w| w as u32),
            height:       video.and_then(|v| v["height"].as_u64()).map(|h| h as u32),
            fps:          video.and_then(|v| parse_fps(v["avg_frame_rate"].as_str())),
            bitrate_kbps: format["bit_rate"].as_str()
                .and_then(|b| b.parse::<u64>().ok())
                .map(|b| b / 1000),
            sample_rate:  audio.and_then(|a| a["sample_rate"].as_str()).and_then(|s| s.parse().ok()),
            channels:     audio.and_then(|a| a["channels"].as_u64()).map(|c| c as u32),
        })
    }

    /// Convert audio to 16kHz mono WAV (required by whisper.cpp)
    pub async fn to_whisper_wav(&self, input: &str, output: Option<&str>) -> Result<ProcessResult> {
        let out = output.map(String::from).unwrap_or_else(|| {
            Path::new(input)
                .with_file_name(format!(
                    "{}_whisper.wav",
                    Path::new(input).file_stem().unwrap_or_default().to_string_lossy()
                ))
                .to_string_lossy()
                .to_string()
        });

        let start = Instant::now();
        self.run_ffmpeg(&["-y", "-i", input, "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", &out]).await?;

        Ok(ProcessResult {
            output_path: out.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
            size_bytes:  std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0),
        })
    }

    /// Extract audio from a video file
    pub async fn extract_audio(&self, input: &str, output: Option<&str>) -> Result<ProcessResult> {
        let out = output.map(String::from).unwrap_or_else(|| {
            Path::new(input).with_extension("wav").to_string_lossy().to_string()
        });

        let start = Instant::now();
        self.run_ffmpeg(&["-y", "-i", input, "-vn", "-acodec", "pcm_s16le", "-ar", "44100", "-ac", "2", &out]).await?;

        Ok(ProcessResult {
            output_path: out.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
            size_bytes:  std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0),
        })
    }

    async fn run_ffmpeg(&self, args: &[&str]) -> Result<()> {
        let output = tokio::process::Command::new(&self.ffmpeg_path)
            .args(args)
            .output()
            .await
            .context("ffmpeg not found")?;

        if !output.status.success() {
            anyhow::bail!("ffmpeg failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }
}

impl Default for MediaProcessor {
    fn default() -> Self { Self::new() }
}

fn parse_fps(fps_str: Option<&str>) -> Option<f64> {
    let s = fps_str?;
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().ok()?;
        let den: f64 = parts[1].parse().ok()?;
        if den > 0.0 { Some(num / den) } else { None }
    } else {
        s.parse().ok()
    }
}
