use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::utils::LogLevel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimulationAddress {
    pub site_id: u32,
    pub application_id: u32,
    pub entity_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdvancedConfig {
    #[serde(default = "LogLevel::default_for_build")]
    pub log_level: LogLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub simulation_address: SimulationAddress,
    #[serde(skip)]
    pub advanced: AdvancedConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            simulation_address: SimulationAddress {
                site_id: 1,
                application_id: 50,
                entity_id: 1,
            },
            advanced: AdvancedConfig::default(),
        }
    }
}

pub type SharedConfig = RwLock<AppConfig>;
