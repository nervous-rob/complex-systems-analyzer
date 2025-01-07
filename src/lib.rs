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

/// Initialize the application with default configuration
pub async fn init() -> Result<SystemManager> {
    todo!("Implement system initialization")
}

/// Version of the Complex Systems Analyzer
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 