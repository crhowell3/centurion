use std::path::PathBuf;
use std::{fs, io};

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("centurion.log")
        .max_log_files(1)
        .build(log_dir)
        .expect("Failed to create log file appender");

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true);

    let stdout_layer = fmt::layer()
        .with_writer(io::stdout)
        .with_target(true)
        .with_line_number(true);

    let level = log_level.to_tracing_level();
    let mut env_filter = EnvFilter::from_default_env().add_directive(level.into());
    let centurion_level = format!("centurion={}", log_level.to_string().to_lowercase());

    env_filter = env_filter.add_directive(centurion_level.parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    tracing::info!("Logging initialized at level: {log_level}");
    tracing::info!("Log directory: {}", log_dir.display());

    Ok(())
}
