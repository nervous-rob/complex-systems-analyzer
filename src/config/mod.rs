use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::core::types::ValidationLevel;

mod validation;
pub use validation::{ValidationResult, validate_config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub system: SystemConfig,
    pub storage: StorageConfig,
    pub compute: ComputeConfig,
    pub ui: crate::ui::UIConfig,
    pub visualization: VisConfig,
    pub logging: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub max_components: usize,
    pub max_relationships: usize,
    pub auto_save_interval: Duration,
    pub validation_level: ValidationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub rocks_db_path: PathBuf,
    pub sqlite_path: PathBuf,
    pub max_cache_size: usize,
    pub backup_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeConfig {
    pub thread_count: usize,
    pub task_queue_size: usize,
    pub max_memory: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisConfig {
    pub renderer_type: String,
    pub max_fps: u32,
    pub antialiasing: bool,
    pub vsync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub log_level: LogLevel,
    pub log_path: PathBuf,
    pub rotation_size: usize,
    pub retention_period: Duration,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Plain,
    Json,
    Custom(String),
}

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    env_overrides: HashMap<String, String>,
}

impl ConfigManager {
    pub async fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            serde_json::from_str(&content)?
        } else {
            Self::default_config()
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            env_overrides: Self::load_env_overrides(),
        })
    }

    pub async fn load_config(&mut self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.config_path).await?;
        let mut config: AppConfig = serde_json::from_str(&content)?;

        // Apply environment overrides
        self.apply_env_overrides(&mut config);

        // Validate the configuration
        let validation_result = validate_config(&config);
        if !validation_result.is_valid {
            return Err(Error::Configuration(format!(
                "Invalid configuration: {:?}",
                validation_result.errors
            )));
        }

        *self.config.write().await = config;
        Ok(())
    }

    pub async fn save_config(&self) -> Result<()> {
        let config = self.config.read().await;
        let content = serde_json::to_string_pretty(&*config)?;
        tokio::fs::write(&self.config_path, content).await?;
        Ok(())
    }

    pub async fn get_config(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&mut self, updates: ConfigUpdate) -> Result<()> {
        let mut config = self.config.write().await;
        updates.apply(&mut config)?;

        // Validate the updated configuration
        let validation_result = validate_config(&config);
        if !validation_result.is_valid {
            return Err(Error::Configuration(format!(
                "Invalid configuration update: {:?}",
                validation_result.errors
            )));
        }

        Ok(())
    }

    fn default_config() -> AppConfig {
        AppConfig {
            system: SystemConfig {
                max_components: 10000,
                max_relationships: 100000,
                auto_save_interval: Duration::from_secs(300),
                validation_level: ValidationLevel::Normal,
            },
            storage: StorageConfig {
                rocks_db_path: PathBuf::from("data/rocks.db"),
                sqlite_path: PathBuf::from("data/sqlite.db"),
                max_cache_size: 1024 * 1024 * 1024, // 1GB
                backup_interval: Duration::from_secs(3600),
            },
            compute: ComputeConfig {
                thread_count: num_cpus::get(),
                task_queue_size: 1000,
                max_memory: 1024 * 1024 * 1024, // 1GB
            },
            ui: crate::ui::UIConfig {
                window_size: (1280, 720),
                theme: crate::ui::Theme::Dark,
                layout: crate::ui::LayoutConfig {
                    layout_type: crate::ui::LayoutType::Force,
                    spacing: 50.0,
                    padding: 20.0,
                },
            },
            visualization: VisConfig {
                renderer_type: "webgl".to_string(),
                max_fps: 60,
                antialiasing: true,
                vsync: true,
            },
            logging: LogConfig {
                log_level: LogLevel::Info,
                log_path: PathBuf::from("logs"),
                rotation_size: 10 * 1024 * 1024, // 10MB
                retention_period: Duration::from_secs(7 * 24 * 3600), // 7 days
                format: LogFormat::Json,
            },
        }
    }

    fn load_env_overrides() -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        
        // Load environment variables with CSA_ prefix
        for (key, value) in std::env::vars() {
            if key.starts_with("CSA_") {
                overrides.insert(key[4..].to_string(), value);
            }
        }

        overrides
    }

    fn apply_env_overrides(&self, config: &mut AppConfig) {
        // Apply environment overrides to configuration
        // This is a simple implementation; in practice, you'd want to handle nested structures
        for (key, value) in &self.env_overrides {
            match key.as_str() {
                "LOG_LEVEL" => {
                    if let Ok(level) = serde_json::from_str(&format!("\"{}\"", value)) {
                        config.logging.log_level = level;
                    }
                }
                "THREAD_COUNT" => {
                    if let Ok(count) = value.parse() {
                        config.compute.thread_count = count;
                    }
                }
                // Add more override handlers as needed
                _ => {}
            }
        }
    }
}

pub struct ConfigUpdate {
    pub system: Option<SystemConfigUpdate>,
    pub storage: Option<StorageConfigUpdate>,
    pub compute: Option<ComputeConfigUpdate>,
    pub ui: Option<UIConfigUpdate>,
    pub visualization: Option<VisConfigUpdate>,
    pub logging: Option<LogConfigUpdate>,
}

impl ConfigUpdate {
    fn apply(self, config: &mut AppConfig) -> Result<()> {
        if let Some(system) = self.system {
            system.apply(&mut config.system)?;
        }
        if let Some(storage) = self.storage {
            storage.apply(&mut config.storage)?;
        }
        if let Some(compute) = self.compute {
            compute.apply(&mut config.compute)?;
        }
        if let Some(ui) = self.ui {
            ui.apply(&mut config.ui)?;
        }
        if let Some(visualization) = self.visualization {
            visualization.apply(&mut config.visualization)?;
        }
        if let Some(logging) = self.logging {
            logging.apply(&mut config.logging)?;
        }
        Ok(())
    }
}

// Update types for partial configuration updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigUpdate {
    pub max_components: Option<usize>,
    pub max_relationships: Option<usize>,
    pub auto_save_interval: Option<Duration>,
    pub validation_level: Option<ValidationLevel>,
}

impl SystemConfigUpdate {
    fn apply(self, config: &mut SystemConfig) -> Result<()> {
        if let Some(max_components) = self.max_components {
            config.max_components = max_components;
        }
        if let Some(max_relationships) = self.max_relationships {
            config.max_relationships = max_relationships;
        }
        if let Some(auto_save_interval) = self.auto_save_interval {
            config.auto_save_interval = auto_save_interval;
        }
        if let Some(validation_level) = self.validation_level {
            config.validation_level = validation_level;
        }
        Ok(())
    }
}

// Similar update types for other config sections...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfigUpdate {
    pub max_cache_size: Option<usize>,
    pub backup_interval: Option<Duration>,
}

impl StorageConfigUpdate {
    fn apply(self, config: &mut StorageConfig) -> Result<()> {
        if let Some(max_cache_size) = self.max_cache_size {
            config.max_cache_size = max_cache_size;
        }
        if let Some(backup_interval) = self.backup_interval {
            config.backup_interval = backup_interval;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeConfigUpdate {
    pub thread_count: Option<usize>,
    pub task_queue_size: Option<usize>,
    pub max_memory: Option<usize>,
}

impl ComputeConfigUpdate {
    fn apply(self, config: &mut ComputeConfig) -> Result<()> {
        if let Some(thread_count) = self.thread_count {
            config.thread_count = thread_count;
        }
        if let Some(task_queue_size) = self.task_queue_size {
            config.task_queue_size = task_queue_size;
        }
        if let Some(max_memory) = self.max_memory {
            config.max_memory = max_memory;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct UIConfigUpdate {
    pub theme: Option<crate::ui::Theme>,
    pub window_size: Option<(u32, u32)>,
    pub layout_type: Option<crate::ui::LayoutType>,
    pub spacing: Option<f32>,
    pub padding: Option<f32>,
}

impl UIConfigUpdate {
    fn apply(self, config: &mut crate::ui::UIConfig) -> Result<()> {
        if let Some(theme) = self.theme {
            config.theme = theme;
        }
        if let Some(window_size) = self.window_size {
            config.window_size = window_size;
        }
        if let Some(layout_type) = self.layout_type {
            config.layout.layout_type = layout_type;
        }
        if let Some(spacing) = self.spacing {
            config.layout.spacing = spacing;
        }
        if let Some(padding) = self.padding {
            config.layout.padding = padding;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisConfigUpdate {
    pub renderer_type: Option<String>,
    pub max_fps: Option<u32>,
    pub antialiasing: Option<bool>,
    pub vsync: Option<bool>,
}

impl VisConfigUpdate {
    fn apply(self, config: &mut VisConfig) -> Result<()> {
        if let Some(renderer_type) = self.renderer_type {
            config.renderer_type = renderer_type;
        }
        if let Some(max_fps) = self.max_fps {
            config.max_fps = max_fps;
        }
        if let Some(antialiasing) = self.antialiasing {
            config.antialiasing = antialiasing;
        }
        if let Some(vsync) = self.vsync {
            config.vsync = vsync;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfigUpdate {
    pub log_level: Option<LogLevel>,
    pub rotation_size: Option<usize>,
    pub retention_period: Option<Duration>,
    pub format: Option<LogFormat>,
}

impl LogConfigUpdate {
    fn apply(self, config: &mut LogConfig) -> Result<()> {
        if let Some(log_level) = self.log_level {
            config.log_level = log_level;
        }
        if let Some(rotation_size) = self.rotation_size {
            config.rotation_size = rotation_size;
        }
        if let Some(retention_period) = self.retention_period {
            config.retention_period = retention_period;
        }
        if let Some(format) = self.format {
            config.format = format;
        }
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            storage: StorageConfig::default(),
            compute: ComputeConfig::default(),
            ui: crate::ui::UIConfig {
                window_size: (1280, 720),
                theme: crate::ui::Theme::Dark,
                layout: crate::ui::LayoutConfig::default(),
            },
            visualization: VisConfig::default(),
            logging: LogConfig::default(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            max_components: 10000,
            max_relationships: 100000,
            auto_save_interval: Duration::from_secs(300),
            validation_level: ValidationLevel::Normal,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            rocks_db_path: PathBuf::from("data/rocks.db"),
            sqlite_path: PathBuf::from("data/sqlite.db"),
            max_cache_size: 1024 * 1024 * 1024, // 1GB
            backup_interval: Duration::from_secs(3600),
        }
    }
}

impl Default for ComputeConfig {
    fn default() -> Self {
        Self {
            thread_count: num_cpus::get(),
            task_queue_size: 1000,
            max_memory: 1024 * 1024 * 1024, // 1GB
        }
    }
}

impl Default for VisConfig {
    fn default() -> Self {
        Self {
            renderer_type: "webgl".to_string(),
            max_fps: 60,
            antialiasing: true,
            vsync: true,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            log_path: PathBuf::from("logs"),
            rotation_size: 10 * 1024 * 1024, // 10MB
            retention_period: Duration::from_secs(7 * 24 * 3600), // 7 days
            format: LogFormat::Json,
        }
    }
} 