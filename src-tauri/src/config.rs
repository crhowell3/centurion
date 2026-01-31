use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::{fs, path::Path};

static CONFIG: OnceLock<Mutex<Option<ScenarioConfiguration>>> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
pub struct Federates {
    pub site_id: u32,
    pub application_id: u32,
    pub entity_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CenturionConfig {
    pub site_id: u32,
    pub application_id: u32,
    pub entity_id: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScenarioConfiguration {
    pub federates: Vec<Federates>,
    pub address: String,
    pub centurion_config: CenturionConfig,
}

#[derive(Debug)]
pub enum ConfigError {
    NotInitialized,
    Io(std::io::Error),
    ParseToml(toml::de::Error),
    InvalidPath(PathBuf),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::ParseToml(err)
    }
}

pub struct Config;

impl Config {
    pub fn init() -> Result<(), ConfigError> {
        CONFIG
            .set(Mutex::new(None))
            .map_err(|_| ConfigError::NotInitialized)
    }

    pub fn set(config: ScenarioConfiguration) -> Result<(), ConfigError> {
        let mutex = CONFIG.get().ok_or(ConfigError::NotInitialized)?;
        let mut guard = mutex.lock().map_err(|_| ConfigError::NotInitialized)?;
        *guard = Some(config);
        Ok(())
    }

    pub fn get() -> Result<ScenarioConfiguration, ConfigError> {
        let mutex = CONFIG.get().ok_or(ConfigError::NotInitialized)?;
        let guard = mutex.lock().map_err(|_| ConfigError::NotInitialized)?;
        guard.clone().ok_or(ConfigError::NotInitialized)
    }
}

pub fn load_config_from_file(path: &Path) -> Result<ScenarioConfiguration, ConfigError> {
    let toml = fs::read_to_string(&path)?;
    let config: ScenarioConfiguration = toml::from_str(&toml)?;

    Ok(config)
}
