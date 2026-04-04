/// commands/services.rs — Hermes API (nightshift) proxy commands
///
/// Routes commands to the Python hermes-api for capabilities not in Rust:
/// - Agent runs, tool calls, approvals, task queue
/// - Dashboard metrics, session history

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name:    String,
    pub status:  String,
    pub uptime_s: Option<u64>,
    pub url:     String,
    pub version: Option<String>,
    pub healthy: bool,
}

#[derive(Debug, Serialize)]
pub struct AllServicesStatus {
    pub hermes_api:    ServiceStatus,
    pub llama_server:  ServiceStatus,
    pub whisper_server: ServiceStatus,
}

/// Get health status of all backend services
#[tauri::command]
pub async fn get_all_services_status(state: State<'_, AppState>) -> Result<AllServicesStatus, String> {
    let (api_ready, llm_ready, whisper_ready) = tokio::join!(
        async { *state.api_ready.read().await },
        async { *state.llm_ready.read().await },
        async { *state.whisper_ready.read().await },
    );

    let hermes_url   = state.hermes_api_url.read().await.clone();
    let llama_url    = state.llama_server_url.read().await.clone();
    let whisper_url  = state.whisper_server_url.read().await.clone();

    Ok(AllServicesStatus {
        hermes_api: ServiceStatus {
            name: "Hermes API".into(),
            status: if api_ready { "running" } else { "stopped" }.into(),
            uptime_s: None,
            url: hermes_url,
            version: None,
            healthy: api_ready,
        },
        llama_server: ServiceStatus {
            name: "LLM Server".into(),
            status: if llm_ready { "running" } else { "stopped" }.into(),
            uptime_s: None,
            url: llama_url,
            version: None,
            healthy: llm_ready,
        },
        whisper_server: ServiceStatus {
            name: "Whisper Server".into(),
            status: if whisper_ready { "running" } else { "stopped" }.into(),
            uptime_s: None,
            url: whisper_url,
            version: None,
            healthy: whisper_ready,
        },
    })
}

/// Restart the Hermes Python API sidecar
#[tauri::command]
pub async fn start_hermes_api(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Trigger sidecar restart via the sidecar manager
    // In production: signal sidecar::restart_hermes_api(&app).await
    *state.api_ready.write().await = false;
    Ok(())
}

#[tauri::command]
pub async fn stop_hermes_api(state: State<'_, AppState>) -> Result<(), String> {
    *state.api_ready.write().await = false;
    Ok(())
}

/// Get dashboard overview from Hermes API
#[tauri::command]
pub async fn get_dashboard_overview(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let url = format!("{}/api/dashboard/overview", state.hermes_api_url.read().await.clone());

    match state.http_client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())
        }
        Ok(resp) => Err(format!("API error: {}", resp.status())),
        Err(_) => Ok(serde_json::json!({
            "total_runs": 0,
            "active_runs": 0,
            "pending_approvals": 0,
            "total_tools": 0,
            "agents_online": 0,
        })),
    }
}

/// Get agent run history
#[tauri::command]
pub async fn get_runs(
    state: State<'_, AppState>,
    limit: Option<u32>,
    status: Option<String>,
) -> Result<serde_json::Value, String> {
    let base = state.hermes_api_url.read().await.clone();
    let mut url = format!("{}/api/runs?limit={}", base, limit.unwrap_or(20));
    if let Some(s) = status {
        url.push_str(&format!("&status={}", s));
    }

    state.http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())
}

/// Get pending tool approvals
#[tauri::command]
pub async fn get_approvals(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let url = format!("{}/api/approvals/pending", state.hermes_api_url.read().await.clone());

    state.http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())
}

/// Approve a pending tool call
#[tauri::command]
pub async fn approve_command(
    state: State<'_, AppState>,
    approval_id: String,
) -> Result<(), String> {
    let url = format!("{}/api/approvals/{}/approve", state.hermes_api_url.read().await.clone(), approval_id);
    state.http_client
        .post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Reject a pending tool call
#[tauri::command]
pub async fn reject_command(
    state: State<'_, AppState>,
    approval_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    let url = format!("{}/api/approvals/{}/reject", state.hermes_api_url.read().await.clone(), approval_id);
    let body = serde_json::json!({ "reason": reason.unwrap_or_default() });
    state.http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
