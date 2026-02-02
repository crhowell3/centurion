//! Core configuration loading and saving logic.
//!
//! This module handles reading and writing the application configuration
//! to and from TOML files.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::config::{AdvancedConfig, AppConfig};

pub async fn load_config(app_handle: &AppHandle) -> AppConfig {
    try_load_or_create_config(app_handle).await.inspect_err(|e| {
        tracing::error!("A critical error occurred during configuration loading: {e}, Using default configuration.");
    }).unwrap_or_default()
}

pub async fn save_config(app_handle: &AppHandle, config: &AppConfig) -> Result<()> {
    #[derive(Serialize)]
    struct ConfigFile<'a> {
        #[serde(flatten)]
        config: &'a AppConfig,
    }

    let config_path = get_config_path(app_handle)?;

    let config_file = ConfigFile { config };

    let toml_string = toml::to_string_pretty(&config_file)
        .context("Failed to serialize configuration to TOML")?;

    tokio::fs::write(&config_path, toml_string)
        .await
        .with_context(|| format!("Failed to write configuration to {}", config_path.display()))?;

    tracing::info!(
        "Configuration saved successfully to {}",
        config_path.display()
    );
    Ok(())
}

fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .context("Failed to get app config directory")?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    Ok(config_dir.join("config.toml"))
}

async fn try_load_or_create_config(app_handle: &AppHandle) -> Result<AppConfig> {
    let config_path = get_config_path(app_handle)?;

    if !config_path.exists() {
        tracing::info!(
            "No config file found at {}. Creating a default one.",
            config_path.display()
        );

        let config = AppConfig::default();
        save_config(app_handle, &config).await.unwrap_or_else(|e| {
            tracing::error!("Failed to save the default config file: {e}");
        });

        return Ok(config);
    }

    let mut config: AppConfig = Figment::new()
        .merge(Serialized::defaults(AppConfig::default()))
        .merge(Toml::file(&config_path))
        .extract()
        .map_err(|e| {
            tracing::error!(
                "Failed to parse config file at {}: {e:#}",
                config_path.display()
            );

            anyhow::Error::new(e).context(format!(
                "Failed to load config from {}",
                config_path.display()
            ))
        })?;

    config.advanced = load_advanced_config(&config_path).await?;

    tracing::info!("Config loaded successfully from {}", config_path.display());

    Ok(config)
}

async fn load_advanced_config(config_path: &PathBuf) -> Result<AdvancedConfig> {
    #[derive(Deserialize)]
    struct ConfigFile {
        #[serde(default)]
        advanced: AdvancedConfig,
    }

    let file_content = tokio::fs::read_to_string(config_path)
        .await
        .context("Failed to read config file for advanced configuration")?;

    let config_file: ConfigFile =
        toml::from_str(&file_content).context("Failed to parse advanced configuration section")?;

    Ok(config_file.advanced)
}
