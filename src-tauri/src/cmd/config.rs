use tauri::{AppHandle, State};

use crate::config::{self, AppConfig, SharedConfig};

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
