use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    Node,
    Agent,
    Process,
    Resource,
    Interface,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentStatus {
    Active,
    Inactive,
    Error,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentState {
    pub current_value: f64,
    pub last_updated: DateTime<Utc>,
    pub history: VecDeque<StateEntry>,
    pub status: ComponentStatus,
}

impl Default for ComponentState {
    fn default() -> Self {
        Self {
            current_value: 0.0,
            last_updated: Utc::now(),
            history: VecDeque::with_capacity(100),
            status: ComponentStatus::Inactive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    Dependency,
    Association,
    Composition,
    Aggregation,
    Flow,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationLevel {
    Strict,
    Normal,
    Lenient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub category: ValidationCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationCategory {
    Structure,
    Consistency,
    Performance,
    Security,
    Custom(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub component_count: usize,
    pub relationship_count: usize,
    pub active_components: usize,
    pub error_components: usize,
    pub last_updated: DateTime<Utc>,
}

impl SystemMetrics {
    pub fn new(
        component_count: usize,
        relationship_count: usize,
        active_components: usize,
        error_components: usize,
    ) -> Self {
        Self {
            component_count,
            relationship_count,
            active_components,
            error_components,
            last_updated: Utc::now(),
        }
    }

    pub fn health_score(&self) -> f64 {
        if self.component_count == 0 {
            return 0.0;
        }
        let active_ratio = self.active_components as f64 / self.component_count as f64;
        let error_ratio = self.error_components as f64 / self.component_count as f64;
        (active_ratio * 100.0) - (error_ratio * 50.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Complexity {
    Constant,      // O(1)
    Logarithmic,   // O(log n)
    Linear,        // O(n)
    Linearithmic,  // O(n log n)
    Quadratic,     // O(n²)
    Cubic,         // O(n³)
    Exponential,   // O(2^n)
    Factorial,     // O(n!)
}

impl Complexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Constant => "O(1)",
            Self::Logarithmic => "O(log n)",
            Self::Linear => "O(n)",
            Self::Linearithmic => "O(n log n)",
            Self::Quadratic => "O(n²)",
            Self::Cubic => "O(n³)",
            Self::Exponential => "O(2^n)",
            Self::Factorial => "O(n!)",
        }
    }
} 


