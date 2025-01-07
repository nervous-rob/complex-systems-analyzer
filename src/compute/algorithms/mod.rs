use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::error::Result;

pub mod centrality;
pub mod community;
pub mod path;

pub use centrality::CentralityAnalysis;
pub use community::CommunityDetection;
pub use path::PathAnalysis;

pub type NodeId = uuid::Uuid;
pub type Weight = f64;
pub type Graph = HashMap<NodeId, Vec<(NodeId, Weight)>>;
pub type Communities = HashMap<NodeId, usize>;
pub type AnalysisResult = HashMap<String, serde_json::Value>;

#[async_trait]
pub trait AnalysisAlgorithm {
    type Input;
    type Parameters;

    async fn execute(&self, input: Self::Input) -> Result<AnalysisResult>;
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
pub struct CentralityParams {
    pub normalize: bool,
    pub weight_threshold: Option<f64>,
}

impl Default for CentralityParams {
    fn default() -> Self {
        Self {
            normalize: true,
            weight_threshold: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityParams {
    pub min_community_size: usize,
    pub max_iterations: usize,
    pub resolution: f64,
}

impl Default for CommunityParams {
    fn default() -> Self {
        Self {
            min_community_size: 3,
            max_iterations: 100,
            resolution: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathParams {
    pub max_path_length: Option<usize>,
    pub weight_function: PathWeightFunction,
}

impl Default for PathParams {
    fn default() -> Self {
        Self {
            max_path_length: None,
            weight_function: PathWeightFunction::Shortest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathWeightFunction {
    Shortest,
    Longest,
    Average,
    Custom(f64),
} 