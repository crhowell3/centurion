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
pub struct Network {
    pub interface_ip: String,
    pub interface_port: u16,
    pub destination_ip: String,
    pub destination_port: u16,
    pub enable_broadcast: bool,
    pub multicast_ttl: u32,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            interface_ip: "0.0.0.0".to_string(),
            interface_port: 3000,
            destination_ip: "0.0.0.0".to_string(),
            destination_port: 3000,
            enable_broadcast: false,
            multicast_ttl: 42,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScenarioConfig {
    pub network: Network,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub simulation_address: SimulationAddress,
    #[serde(skip)]
    pub advanced: AdvancedConfig,
    pub scenario_config: ScenarioConfig,
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
            scenario_config: ScenarioConfig::default(),
        }
    }
}

pub type SharedConfig = RwLock<AppConfig>;
