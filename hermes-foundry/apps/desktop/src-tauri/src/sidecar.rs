/// sidecar.rs — manages all child process sidecars:
///   - hermes-api     (FastAPI Python, port 8000)
///   - llama-server   (llama.cpp, port 8080)
///   - whisper-server (whisper.cpp, port 8090)

use anyhow::{anyhow, Result};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tracing::{error, info, warn};

use crate::state::{AppState, PORT_HERMES_API, PORT_LLAMA_SERVER, PORT_WHISPER_SERVER};

/// Starts all sidecar processes in the correct order, waits for health
pub async fn start_all_sidecars(app: &AppHandle) -> Result<()> {
    // Sidecars start in parallel — each has its own retry logic
    let (api_res, llm_res, whisper_res) = tokio::join!(
        start_hermes_api(app),
        start_llama_server(app),
        start_whisper_server(app),
    );

    if let Err(e) = api_res {
        warn!("hermes-api failed to start: {} — continuing without it", e);
    }
    if let Err(e) = llm_res {
        warn!("llama-server failed to start: {} — continuing without it", e);
    }
    if let Err(e) = whisper_res {
        warn!("whisper-server failed to start: {} — continuing without it", e);
    }

    Ok(())
}

/// Gracefully stop all sidecars
pub async fn stop_all_sidecars(app: &AppHandle) {
    let state = app.state::<AppState>();
    let pids = state.sidecar_pids.lock().await;
    info!("Stopping {} sidecar(s)...", pids.len());
    // Tauri shell plugin handles cleanup when handles drop
}

/// Start the Hermes nightshift FastAPI backend
async fn start_hermes_api(app: &AppHandle) -> Result<()> {
    info!("Starting hermes-api on port {}", PORT_HERMES_API);
    let state = app.state::<AppState>();

    let shell = app.shell();
    let (mut rx, _child) = shell
        .sidecar("hermes-api")?
        .args(["--host", "127.0.0.1", "--port", &PORT_HERMES_API.to_string()])
        .spawn()?;

    // Wait for health endpoint
    let url = format!("http://127.0.0.1:{}/health", PORT_HERMES_API);
    if wait_for_health(&url, 30).await {
        *state.api_ready.write().await = true;
        info!("hermes-api is ready");
    } else {
        error!("hermes-api health check timed out");
    }

    // Drain stdout/stderr to avoid blocking
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
                tracing::debug!("[hermes-api] {}", String::from_utf8_lossy(&line));
            }
        }
    });

    Ok(())
}

/// Start llama-server (atomic-llama-cpp-turboquant)
async fn start_llama_server(app: &AppHandle) -> Result<()> {
    info!("Starting llama-server on port {}", PORT_LLAMA_SERVER);
    let state = app.state::<AppState>();

    // Determine models dir
    let models_dir = get_models_dir(app);
    let models_path = models_dir.to_string_lossy().to_string();

    let shell = app.shell();
    let (mut rx, _child) = shell
        .sidecar("llama-server")?
        .args([
            "--host", "127.0.0.1",
            "--port", &PORT_LLAMA_SERVER.to_string(),
            "--models-dir", &models_path,
            "--parallel", "2",
            "--cont-batching",
            "--log-disable",
        ])
        .spawn()?;

    let url = format!("http://127.0.0.1:{}/health", PORT_LLAMA_SERVER);
    if wait_for_health(&url, 20).await {
        *state.llm_ready.write().await = true;
        info!("llama-server is ready");
    } else {
        warn!("llama-server not ready (no models loaded yet — normal on first launch)");
        *state.llm_ready.write().await = true; // Mark ready even without model
    }

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
                tracing::debug!("[llama-server] {}", String::from_utf8_lossy(&line));
            }
        }
    });

    Ok(())
}

/// Start whisper-server (whisper.cpp)
async fn start_whisper_server(app: &AppHandle) -> Result<()> {
    info!("Starting whisper-server on port {}", PORT_WHISPER_SERVER);
    let state = app.state::<AppState>();

    let models_dir = get_models_dir(app);
    let model_path = models_dir.join("ggml-base.en.bin");

    // Don't start whisper-server if base model isn't downloaded yet
    if !model_path.exists() {
        warn!("Whisper base model not found at {:?} — voice input disabled until model is downloaded", model_path);
        return Ok(());
    }

    let shell = app.shell();
    let (mut rx, _child) = shell
        .sidecar("whisper-server")?
        .args([
            "--host", "127.0.0.1",
            "--port", &PORT_WHISPER_SERVER.to_string(),
            "-m", &model_path.to_string_lossy(),
            "-t", "4",
        ])
        .spawn()?;

    let url = format!("http://127.0.0.1:{}/", PORT_WHISPER_SERVER);
    if wait_for_health(&url, 15).await {
        *state.whisper_ready.write().await = true;
        info!("whisper-server is ready");
    }

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
                tracing::debug!("[whisper] {}", String::from_utf8_lossy(&line));
            }
        }
    });

    Ok(())
}

/// Poll a health URL until it returns 200 or we time out
async fn wait_for_health(url: &str, timeout_secs: u64) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);

    while tokio::time::Instant::now() < deadline {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => return true,
            _ => {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    false
}

/// Returns the path to the Hermes models directory, creating it if needed
pub fn get_models_dir(app: &AppHandle) -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let models_dir = home.join(".hermes").join("models");
    let _ = std::fs::create_dir_all(&models_dir);
    models_dir
}
