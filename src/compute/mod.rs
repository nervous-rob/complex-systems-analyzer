use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Duration;
use rayon::ThreadPool;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};

mod engine;
mod task;
pub mod algorithms;

pub use engine::ComputeEngine;
pub use task::{ComputeTask, TaskHandle, ComputeResult};
pub use algorithms::{AnalysisAlgorithm, CentralityAnalysis, CommunityDetection, PathAnalysis};

#[derive(Debug, Clone)]
pub struct ComputeConfig {
    pub thread_count: usize,
    pub task_queue_size: usize,
    pub max_memory: usize,
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

#[derive(Debug, Clone)]
pub struct ComputeStats {
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_task_duration: Duration,
    pub memory_usage: usize,
}

// Analysis types and configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Centrality(CentralityType),
    Community(CommunityType),
    Path(PathType),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CentralityType {
    Degree,
    Betweenness,
    Closeness,
    Eigenvector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunityType {
    Louvain,
    LabelPropagation,
    Infomap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    ShortestPath,
    AllPaths,
    CriticalPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub analysis_type: AnalysisType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub constraints: AnalysisConstraints,
    #[serde(with = "serde_duration")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConstraints {
    pub max_iterations: Option<usize>,
    pub convergence_threshold: Option<f64>,
    pub max_memory: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub analysis_type: AnalysisType,
    pub result_data: HashMap<String, serde_json::Value>,
    #[serde(with = "serde_duration")]
    pub computation_time: Duration,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Helper module for serializing Duration
mod serde_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

// Add conversions between compute and algorithm types
impl From<CentralityType> for algorithms::CentralityType {
    fn from(ct: CentralityType) -> Self {
        match ct {
            CentralityType::Degree => algorithms::CentralityType::Degree,
            CentralityType::Betweenness => algorithms::CentralityType::Betweenness,
            CentralityType::Closeness => algorithms::CentralityType::Closeness,
            CentralityType::Eigenvector => algorithms::CentralityType::Eigenvector,
        }
    }
}

impl From<CommunityType> for algorithms::CommunityType {
    fn from(ct: CommunityType) -> Self {
        match ct {
            CommunityType::Louvain => algorithms::CommunityType::Louvain,
            CommunityType::LabelPropagation => algorithms::CommunityType::LabelPropagation,
            CommunityType::Infomap => algorithms::CommunityType::Infomap,
        }
    }
}

impl From<PathType> for algorithms::PathType {
    fn from(pt: PathType) -> Self {
        match pt {
            PathType::ShortestPath => algorithms::PathType::ShortestPath,
            PathType::AllPaths => algorithms::PathType::AllPaths,
            PathType::CriticalPath => algorithms::PathType::CriticalPath,
        }
    }
} 