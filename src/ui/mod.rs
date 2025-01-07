use std::sync::Arc;
use std::sync::mpsc;
use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};
use crate::error::Result;

mod app;
mod state;
mod views;
mod widgets;

pub use app::App;
pub use state::AppState;

// UI Configuration Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    Force,
    Grid,
    Circular,
    Hierarchical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub layout_type: LayoutType,
    pub spacing: f32,
    pub padding: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::Force,
            spacing: 50.0,
            padding: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub window_size: (u32, u32),
    pub theme: Theme,
    pub layout: LayoutConfig,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            window_size: (1280, 720),
            theme: Theme::System,
            layout: LayoutConfig::default(),
        }
    }
}

// UI Update Types
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

// UI Bridge
pub struct UIBridge {
    state: Arc<AppState>,
    event_sender: mpsc::Sender<UIEvent>,
}

impl UIBridge {
    pub fn new(config: UIConfig) -> Self {
        let (event_sender, _event_receiver) = mpsc::channel();
        let state = Arc::new(AppState::new(config));
        
        Self {
            state,
            event_sender,
        }
    }

    pub fn initialize(&self) -> Result<()> {
        Ok(())
    }

    pub fn handle_command(&self, _command: UICommand) -> Result<CommandResponse> {
        Ok(CommandResponse {
            success: true,
            data: None,
            error: None,
        })
    }

    pub fn update_view(&self, _update: ViewUpdate) -> Result<()> {
        Ok(())
    }

    pub fn register_callback(&self, _event: UIEvent, _callback: Box<dyn Fn(UIEvent)>) -> Result<()> {
        Ok(())
    }
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