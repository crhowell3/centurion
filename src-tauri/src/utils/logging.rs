use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, EnumIter,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[must_use]
    pub fn to_tracing_level(self) -> Level {
        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }

    #[must_use]
    pub fn default_for_build() -> Self {
        #[cfg(debug_assertions)]
        {
            LogLevel::Trace
        }
        #[cfg(not(debug_assertions))]
        {
            LogLevel::Info
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::default_for_build()
    }
}

pub fn init_logging(log_dir: &PathBuf, log_level: LogLevel) -> Result<(), String> {
    fs::create_dir_all(log_dir).map_err(|e| format!("Failed to create log directory: {e}"))?;

    let level = log_level.to_tracing_level();
    let mut env_filter = EnvFilter::from_default_env().add_directive(level.into());
    let centurion_level = format!("centurion={}", log_level.to_string().to_lowercase());

    env_filter = env_filter.add_directive(centurion_level.parse().unwrap());

    tracing_subscriber::registry().with(env_filter).init();

    Ok(())
}
