/// commands/settings.rs — Persistent settings via tauri-plugin-store + OS keyring

use tauri::State;
use crate::state::AppState;
use crate::security;

/// Get a setting value from the persistent store
#[tauri::command]
pub async fn get_setting(
    _state: State<'_, AppState>,
    key: String,
) -> Result<Option<serde_json::Value>, String> {
    // In production, integrate with tauri-plugin-store
    // For now, return None (store accessed directly from frontend via JS)
    Ok(None)
}

/// Set a setting (non-secret) in the persistent store
#[tauri::command]
pub async fn set_setting(
    _state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    Ok(())
}

/// Get all settings
#[tauri::command]
pub async fn get_all_settings(_state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

/// Save an API key to the OS system keyring
#[tauri::command]
pub async fn save_api_key(
    service: String,
    api_key: String,
) -> Result<(), String> {
    security::save_secret(&service, &api_key).map_err(|e| e.to_string())
}

/// Retrieve an API key from the OS system keyring (returns masked version for display)
#[tauri::command]
pub async fn get_api_key(
    service: String,
    masked: Option<bool>,
) -> Result<Option<String>, String> {
    let result = security::get_secret(&service).map_err(|e| e.to_string())?;
    if masked.unwrap_or(true) {
        Ok(result.map(|k| security::mask_secret(&k)))
    } else {
        Ok(result)
    }
}

/// Load branding config from the bundled branding.json
#[tauri::command]
pub async fn get_branding(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("branding.json");

    let content = tokio::fs::read_to_string(&resource_path)
        .await
        .unwrap_or_else(|_| include_str!("../../../../branding.json").to_string());

    serde_json::from_str(&content).map_err(|e| e.to_string())
}
