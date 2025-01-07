use std::path::PathBuf;
use tracing_subscriber::{
    fmt,
    EnvFilter,
    Registry,
    layer::SubscriberExt,
};
use tracing_appender::non_blocking::WorkerGuard;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: LogLevel,
    pub file_path: Option<PathBuf>,
    pub rotation: LogRotation,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    pub max_size: usize,      // Maximum size in bytes
    pub max_files: usize,     // Maximum number of files to keep
    pub rotation_hours: u32,  // Rotate files every N hours
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Plain,
    Json,
    Compact,
}

pub struct LogManager {
    config: LogConfig,
    _guard: Option<WorkerGuard>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_path: None,
            rotation: LogRotation {
                max_size: 50 * 1024 * 1024,  // 50MB
                max_files: 5,
                rotation_hours: 24,
            },
            format: LogFormat::Plain,
        }
    }
}

impl LogManager {
    pub fn new(config: LogConfig) -> Result<Self> {
        let (non_blocking, guard) = if let Some(path) = &config.file_path {
            let (writer, guard) = tracing_appender::non_blocking(
                tracing_appender::rolling::daily(path, "csa.log")
            );
            (Some(writer), Some(guard))
        } else {
            (None, None)
        };

        let env_filter = EnvFilter::from_default_env()
            .add_directive(match config.level {
                LogLevel::Error => format!("error").parse().unwrap(),
                LogLevel::Warn => format!("warn").parse().unwrap(),
                LogLevel::Info => format!("info").parse().unwrap(),
                LogLevel::Debug => format!("debug").parse().unwrap(),
                LogLevel::Trace => format!("trace").parse().unwrap(),
            });

        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);

        let subscriber = Registry::default()
            .with(env_filter)
            .with(fmt_layer);

        if let Some(writer) = non_blocking {
            let file_layer = fmt::layer()
                .with_writer(writer)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true);

            tracing::subscriber::set_global_default(subscriber.with(file_layer))
                .map_err(|e| crate::error::Error::Runtime(format!("Failed to set subscriber: {}", e)))?;
        } else {
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| crate::error::Error::Runtime(format!("Failed to set subscriber: {}", e)))?;
        }

        Ok(Self {
            config,
            _guard: guard,
        })
    }
} 