use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::config::{self, AppConfig, ScenarioConfig, SharedConfig};

#[tauri::command]
pub async fn load_scenario_config(app: tauri::AppHandle) -> Result<ScenarioConfig, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("config", &["toml"])
        .blocking_pick_file();

    if let Some(path) = file {
        let path = path.into_path().map_err(|_| "Invalid file path")?;
        let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        toml::from_str(&contents).map_err(|e| e.to_string())
    } else {
        Err("No file selected".to_string())
    }
}

#[tauri::command]
pub async fn get_config(config_state: State<'_, SharedConfig>) -> Result<AppConfig, String> {
    Ok(config_state.read().await.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    app_handle: AppHandle,
    config_state: State<'_, SharedConfig>,
) -> Result<(), String> {
    config::save_config(&app_handle, &config)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save config file: {e}");
            e.to_string()
        })?;

    {
        let mut config_guard = config_state.write().await;
        *config_guard = config.clone();
    }

    Ok(())
}
