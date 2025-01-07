pub mod compute;
pub mod config;
pub mod core;
pub mod error;
pub mod events;
pub mod io;
pub mod logging;
pub mod runtime;
pub mod storage;
pub mod util;
pub mod validation;

// Core functionality modules
pub mod graph;
pub mod visualization;

// UI and state management
pub mod ui;

// Re-export commonly used types
pub use crate::core::{Component, ComponentType, Relationship, RelationshipType, System};
pub use crate::error::{Error, Result};
pub use crate::core::SystemManager;
pub use crate::compute::algorithms::{AnalysisAlgorithm, CentralityAnalysis};
pub use crate::ui::{App, UIConfig};

use std::sync::Arc;

/// Initialize the application with default configuration
pub async fn init() -> Result<SystemManager> {
    // Initialize storage
    let storage_config = storage::StorageConfig::default();
    let storage = Arc::new(storage::StorageManager::new(storage_config)?);
    storage.init_storage().await?;

    // Initialize compute engine
    let compute_config = compute::ComputeConfig::default();
    let compute = Arc::new(compute::ComputeEngine::new(compute_config)?);

    // Initialize event bus
    let event_bus = Arc::new(events::EventBus::new());

    // Create and return system manager
    Ok(SystemManager::new(storage, compute, event_bus))
}

/// Version of the Complex Systems Analyzer
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 