use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::core::{System, Component, Relationship};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    DataCorruption,
    Concurrency,
    Validation,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

pub struct ValidationRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub check_function: Arc<dyn Fn(&ValidationContext) -> bool + Send + Sync>,
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub system: Option<Arc<System>>,
    pub component: Option<Arc<Component>>,
    pub relationship: Option<Arc<Relationship>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub rule_id: Uuid,
    pub message: String,
    pub severity: ValidationSeverity,
    pub context: ValidationContext,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub rule_id: Uuid,
    pub message: String,
    pub context: ValidationContext,
}

#[derive(Debug, Default)]
pub struct ValidationMetrics {
    pub total_validations: usize,
    pub passed_validations: usize,
    pub failed_validations: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub metrics: ValidationMetrics,
}

#[derive(Debug)]
pub struct SystemError {
    pub error_type: ErrorType,
    pub message: String,
    pub source: Option<Box<dyn Error + Send + Sync>>,
    pub context: ErrorContext,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct ErrorContext {
    pub system_id: Option<Uuid>,
    pub component_id: Option<Uuid>,
    pub relationship_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ErrorHandlingResult {
    pub resolved: bool,
    pub recovery_action_taken: String,
    pub new_errors: Vec<SystemError>,
}

#[derive(Debug, Clone, Copy)]
pub enum RecoveryStrategy {
    Retry,
    Rollback,
    Compensate,
    Ignore,
}

pub trait Validator: Send + Sync {
    fn validate(&self, context: &ValidationContext) -> ValidationResult;
    fn get_severity(&self) -> ValidationSeverity;
    fn get_validation_rules(&self) -> Vec<ValidationRule>;
}

pub trait ErrorHandler: Send + Sync {
    fn handle_error(&self, error: &SystemError) -> ErrorHandlingResult;
    fn can_handle(&self, error: &SystemError) -> bool;
    fn get_recovery_strategy(&self) -> RecoveryStrategy;
}

pub struct ValidationEngine {
    validators: Vec<Box<dyn Validator>>,
    error_handlers: HashMap<ErrorType, Box<dyn ErrorHandler>>,
}

impl ValidationEngine {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            error_handlers: HashMap::new(),
        }
    }

    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }

    pub fn add_error_handler(&mut self, error_type: ErrorType, handler: Box<dyn ErrorHandler>) {
        self.error_handlers.insert(error_type, handler);
    }

    pub fn validate_system(&self, system: &System) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metrics: ValidationMetrics::default(),
        };

        let context = ValidationContext {
            system: Some(Arc::new(system.clone())),
            component: None,
            relationship: None,
            metadata: HashMap::new(),
        };

        for validator in &self.validators {
            let validation = validator.validate(&context);
            result.errors.extend(validation.errors);
            result.warnings.extend(validation.warnings);
            result.metrics.total_validations += validation.metrics.total_validations;
            result.metrics.passed_validations += validation.metrics.passed_validations;
            result.metrics.failed_validations += validation.metrics.failed_validations;
            result.metrics.warning_count += validation.metrics.warning_count;
            result.metrics.error_count += validation.metrics.error_count;
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    pub fn handle_error(&self, error: &SystemError) -> ErrorHandlingResult {
        if let Some(handler) = self.error_handlers.get(&error.error_type) {
            handler.handle_error(error)
        } else {
            ErrorHandlingResult {
                resolved: false,
                recovery_action_taken: "No handler found".to_string(),
                new_errors: vec![],
            }
        }
    }

    pub fn get_validation_metrics(&self) -> ValidationMetrics {
        ValidationMetrics::default() // TODO: Implement actual metrics collection
    }
}

// Re-export commonly used items
pub use self::validators::{SystemIntegrityValidator, ComponentValidator, RelationshipValidator};
pub use self::handlers::{DataCorruptionHandler, ConcurrencyHandler};

// Submodules
mod validators;
mod handlers; 