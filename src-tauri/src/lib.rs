//! # centurion - generic simulation management application
//!
//! ## Architecture
//!
//! - **`cmd`**: Tauri command handlers for communication between the frontend and backend.

use tauri::Manager;
use tauri::async_runtime::spawn as tauri_spawn;

use std::sync::Mutex;

pub mod cmd;
pub mod config;
pub mod core;
pub mod utils;

/// Runs the Tauri application and executes the setup logic.
///
/// # Panics
/// - May panic if tauri fails to generate context
///
/// These are intentional as the application cannot function without a Tauri runtime.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let log_dir = app
                .path()
                .app_log_dir()
                .expect("Failed to get app log directory");

            let handle = app.handle().clone();

            tauri_spawn(async move {
                let app_config = config::load_config(&handle).await;

                utils::init_logging(&log_dir, app_config.advanced.log_level).unwrap_or_else(|e| {
                    eprintln!("Failed to initialize logging: {e}");
                });
            });

            Ok(())
        })
        .manage(core::AppState {
            simulation_state: Mutex::new(core::SimulationState::Stopped),
            request_ids: Mutex::new(core::RequestIds::new()),
        })
        .invoke_handler(tauri::generate_handler![
            cmd::config::get_config,
            cmd::config::save_config,
            cmd::config::load_scenario_config,
            cmd::transmit::send_siman_pdu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
