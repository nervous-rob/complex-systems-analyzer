use super::{ErrorHandler, SystemError, ErrorHandlingResult, RecoveryStrategy};

pub struct DataCorruptionHandler;

impl ErrorHandler for DataCorruptionHandler {
    fn handle_error(&self, error: &SystemError) -> ErrorHandlingResult {
        ErrorHandlingResult {
            resolved: false,
            recovery_action_taken: "Data corruption detected, initiating recovery".to_string(),
            new_errors: Vec::new(),
        }
    }

    fn can_handle(&self, error: &SystemError) -> bool {
        matches!(error.error_type, super::ErrorType::DataCorruption)
    }

    fn get_recovery_strategy(&self) -> RecoveryStrategy {
        RecoveryStrategy::Rollback
    }
}

pub struct ConcurrencyHandler;

impl ErrorHandler for ConcurrencyHandler {
    fn handle_error(&self, error: &SystemError) -> ErrorHandlingResult {
        ErrorHandlingResult {
            resolved: true,
            recovery_action_taken: "Retrying operation after concurrency conflict".to_string(),
            new_errors: Vec::new(),
        }
    }

    fn can_handle(&self, error: &SystemError) -> bool {
        matches!(error.error_type, super::ErrorType::Concurrency)
    }

    fn get_recovery_strategy(&self) -> RecoveryStrategy {
        RecoveryStrategy::Retry
    }
} 