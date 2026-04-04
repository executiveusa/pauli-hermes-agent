/// hermes-media — FFmpeg-powered audio/video processing
///
/// Uses FFmpeg subprocess. The ffmpeg binary must be in PATH or bundled as a Tauri sidecar.

pub mod processor;
pub use processor::MediaProcessor;
