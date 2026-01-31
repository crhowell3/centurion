use tauri::Builder;

use std::sync::Mutex;

mod app_data;
mod commands;
mod config;

use app_data::*;
use commands::*;
use config::*;

pub const VERSION_AND_GIT_HASH: &str = env!("VERSION_AND_GIT_HASH");

pub fn run_cli(path: std::path::PathBuf) -> Result<(), ConfigError> {
    let config = load_config_from_file(&path.as_path())?;
    Config::set(config)?;
    println!("Config loaded from CLI");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
///
/// # Panics
/// - May panic if tauri fails to generate context
///
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppData {
            simulation_state: Mutex::new(SimulationState::Stopped),
            request_ids: Mutex::new(RequestIds::new()),
        })
        .invoke_handler(tauri::generate_handler![
            send_startup,
            send_terminate,
            send_standby,
            send_restart,
            get_version,
            get_centurion_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
