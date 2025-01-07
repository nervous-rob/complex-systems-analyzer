use std::path::Path;
use super::AppConfig;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, message: impl Into<String>) {
        self.is_valid = false;
        self.errors.push(message.into());
    }

    fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
}

pub fn validate_config(config: &AppConfig) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Validate system configuration
    validate_system_config(&config.system, &mut result);

    // Validate storage configuration
    validate_storage_config(&config.storage, &mut result);

    // Validate compute configuration
    validate_compute_config(&config.compute, &mut result);

    // Validate UI configuration
    validate_ui_config(&config.ui, &mut result);

    // Validate visualization configuration
    validate_vis_config(&config.visualization, &mut result);

    // Validate logging configuration
    validate_logging_config(&config.logging, &mut result);

    result
}

fn validate_system_config(config: &super::SystemConfig, result: &mut ValidationResult) {
    if config.max_components == 0 {
        result.add_error("max_components must be greater than 0");
    }

    if config.max_relationships == 0 {
        result.add_error("max_relationships must be greater than 0");
    }

    if config.max_relationships < config.max_components {
        result.add_warning("max_relationships is less than max_components, which might be restrictive");
    }

    if config.auto_save_interval.as_secs() < 60 {
        result.add_warning("auto_save_interval is less than 60 seconds, which might impact performance");
    }
}

fn validate_storage_config(config: &super::StorageConfig, result: &mut ValidationResult) {
    // Validate paths
    if !is_valid_path(&config.rocks_db_path) {
        result.add_error(format!(
            "Invalid RocksDB path: {}",
            config.rocks_db_path.display()
        ));
    }

    if !is_valid_path(&config.sqlite_path) {
        result.add_error(format!(
            "Invalid SQLite path: {}",
            config.sqlite_path.display()
        ));
    }

    // Validate cache size
    const MIN_CACHE_SIZE: usize = 64 * 1024 * 1024; // 64MB
    if config.max_cache_size < MIN_CACHE_SIZE {
        result.add_error(format!(
            "max_cache_size must be at least {}MB",
            MIN_CACHE_SIZE / (1024 * 1024)
        ));
    }

    // Validate backup interval
    if config.backup_interval.as_secs() < 300 {
        result.add_warning("backup_interval is less than 5 minutes, which might impact performance");
    }
}

fn validate_compute_config(config: &super::ComputeConfig, result: &mut ValidationResult) {
    let available_threads = num_cpus::get();

    if config.thread_count == 0 {
        result.add_error("thread_count must be greater than 0");
    } else if config.thread_count > available_threads * 2 {
        result.add_warning(format!(
            "thread_count ({}) is more than twice the number of available CPU threads ({})",
            config.thread_count, available_threads
        ));
    }

    if config.task_queue_size < 10 {
        result.add_error("task_queue_size must be at least 10");
    }

    const MIN_MEMORY: usize = 256 * 1024 * 1024; // 256MB
    if config.max_memory < MIN_MEMORY {
        result.add_error(format!(
            "max_memory must be at least {}MB",
            MIN_MEMORY / (1024 * 1024)
        ));
    }
}

fn validate_ui_config(config: &super::UIConfig, result: &mut ValidationResult) {
    let valid_themes = ["light", "dark", "system"];
    if !valid_themes.contains(&config.theme.as_str()) {
        result.add_error(format!(
            "Invalid theme '{}'. Must be one of: {:?}",
            config.theme, valid_themes
        ));
    }

    let (width, height) = config.window_size;
    if width < 800 || height < 600 {
        result.add_warning("Window size is smaller than recommended minimum (800x600)");
    }
}

fn validate_vis_config(config: &super::VisConfig, result: &mut ValidationResult) {
    let valid_renderers = ["webgl", "canvas", "svg"];
    if !valid_renderers.contains(&config.renderer_type.as_str()) {
        result.add_error(format!(
            "Invalid renderer_type '{}'. Must be one of: {:?}",
            config.renderer_type, valid_renderers
        ));
    }

    if config.max_fps == 0 {
        result.add_error("max_fps must be greater than 0");
    } else if config.max_fps > 144 {
        result.add_warning("max_fps is higher than 144, which might not be necessary");
    }
}

fn validate_logging_config(config: &super::LogConfig, result: &mut ValidationResult) {
    if !is_valid_path(&config.log_path) {
        result.add_error(format!(
            "Invalid log path: {}",
            config.log_path.display()
        ));
    }

    const MIN_ROTATION_SIZE: usize = 1024 * 1024; // 1MB
    if config.rotation_size < MIN_ROTATION_SIZE {
        result.add_error(format!(
            "rotation_size must be at least {}MB",
            MIN_ROTATION_SIZE / (1024 * 1024)
        ));
    }

    if config.retention_period.as_secs() < 24 * 3600 {
        result.add_warning("retention_period is less than 24 hours");
    }
}

fn is_valid_path(path: &Path) -> bool {
    if path.is_absolute() {
        path.parent().map_or(false, |p| p.exists())
    } else {
        true // Relative paths are considered valid
    }
} 