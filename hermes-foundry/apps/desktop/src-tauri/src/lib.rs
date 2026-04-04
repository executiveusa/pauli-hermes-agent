use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod commands;
mod sidecar;
mod state;
mod security;

pub use state::AppState;

pub fn run() {
    // Init structured logging
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    tracing::info!("Hermes Foundry starting up...");

    tauri::Builder::default()
        // ── Plugins ────────────────────────────────────────────────────────
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // ── Managed state ──────────────────────────────────────────────────
        .manage(AppState::default())
        // ── Setup hook ─────────────────────────────────────────────────────
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Show main window after setup completes (avoids flash of unstyled content)
            let main_window = app.get_webview_window("main").unwrap();

            // Start all sidecar services
            let handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                match sidecar::start_all_sidecars(&handle_clone).await {
                    Ok(_) => {
                        tracing::info!("All sidecars started successfully");
                        // Show window once services are up
                        let _ = main_window.show();
                    }
                    Err(e) => {
                        tracing::error!("Sidecar startup error: {}", e);
                        // Show window anyway so user can diagnose
                        let _ = main_window.show();
                    }
                }
            });

            Ok(())
        })
        // ── Command handlers ───────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // Chat / agent
            commands::chat::send_message,
            commands::chat::stream_message,
            commands::chat::get_conversation_history,
            commands::chat::clear_conversation,
            commands::chat::stop_generation,
            commands::chat::list_conversations,
            // Voice (whisper)
            commands::voice::start_recording,
            commands::voice::stop_recording,
            commands::voice::transcribe_file,
            commands::voice::list_voice_models,
            commands::voice::get_voice_server_status,
            // Media (FFmpeg)
            commands::media::process_audio,
            commands::media::process_video,
            commands::media::get_media_info,
            commands::media::extract_audio_from_video,
            commands::media::convert_audio_for_whisper,
            // LLM (llama-server)
            commands::llm::list_local_models,
            commands::llm::get_llm_server_status,
            commands::llm::load_local_model,
            commands::llm::unload_local_model,
            commands::llm::complete,
            commands::llm::scan_models_dir,
            // Hermes services (nightshift)
            commands::services::get_all_services_status,
            commands::services::start_hermes_api,
            commands::services::stop_hermes_api,
            commands::services::get_dashboard_overview,
            commands::services::get_runs,
            commands::services::get_approvals,
            commands::services::approve_command,
            commands::services::reject_command,
            // Settings / security
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::settings::save_api_key,
            commands::settings::get_api_key,
            commands::settings::get_branding,
        ])
        // ── Window close → stop sidecars ───────────────────────────────────
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    sidecar::stop_all_sidecars(&app).await;
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Hermes Foundry");
}
