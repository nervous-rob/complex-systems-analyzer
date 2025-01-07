use std::sync::Arc;
use std::sync::mpsc;
use serde_json::Value as JsonValue;
use crate::error::{Error, Result};

mod app;
mod state;
mod views;
mod widgets;

pub use app::App;
pub use state::AppState;

#[derive(Debug, Clone)]
pub struct ViewUpdate {
    pub component_updates: Vec<ComponentUpdate>,
    pub layout_updates: Option<LayoutUpdate>,
}

#[derive(Debug, Clone)]
pub struct ComponentUpdate {
    pub id: String,
    pub properties: JsonValue,
}

#[derive(Debug, Clone)]
pub struct LayoutUpdate {
    pub layout_type: LayoutType,
    pub parameters: LayoutParams,
}

pub struct UIBridge {
    state: Arc<AppState>,
    event_sender: mpsc::Sender<UIEvent>,
}

impl UIBridge {
    pub fn new(config: UIConfig) -> Self {
        todo!("Implement UIBridge::new")
    }

    pub fn initialize(&self) -> Result<()> {
        todo!("Implement initialization")
    }

    pub fn handle_command(&self, command: UICommand) -> Result<CommandResponse> {
        todo!("Implement command handling")
    }

    pub fn update_view(&self, update: ViewUpdate) -> Result<()> {
        todo!("Implement view updates")
    }

    pub fn register_callback(&self, event: UIEvent, callback: Box<dyn Fn(UIEvent)>) -> Result<()> {
        todo!("Implement callback registration")
    }
}

#[derive(Debug, Clone)]
pub struct UIConfig {
    pub window_size: (u32, u32),
    pub theme: Theme,
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub layout_type: LayoutType,
    pub parameters: LayoutParams,
}

#[derive(Debug, Clone)]
pub enum LayoutType {
    ForceDirected,
    Hierarchical,
    Circular,
}

#[derive(Debug, Clone)]
pub struct LayoutParams {
    pub spacing: f32,
    pub iterations: u32,
    pub force_strength: f32,
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    GraphUpdated,
    SelectionChanged(Vec<String>),
    ViewportChanged,
    AnalysisStarted,
    AnalysisCompleted,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub include_centrality: bool,
    pub include_clustering: bool,
    pub include_paths: bool,
    pub damping_factor: f32,
    pub max_iterations: u32,
    pub convergence_threshold: f32,
}

#[derive(Debug)]
pub enum UICommand {
    RunAnalysis(AnalysisConfig),
    UpdateComponent(ComponentUpdate),
    ExportGraph(String),
    ImportGraph(String),
}

#[derive(Debug)]
pub struct CommandResponse {
    pub success: bool,
    pub data: Option<JsonValue>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AnalysisResult {
    Centrality(Vec<(String, f64)>),
    Clustering(Vec<Vec<String>>),
    Paths(Vec<(String, String, Vec<String>)>),
} 