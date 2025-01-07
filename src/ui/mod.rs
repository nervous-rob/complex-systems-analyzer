use std::sync::Arc;
use std::sync::mpsc::{self, SendError};
use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};
use crate::error::Result;
use parking_lot::RwLock;
use crate::visualization::Visualization;

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
    pub window_title: String,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            window_size: (1280, 720),
            theme: Theme::System,
            layout: LayoutConfig::default(),
            window_title: "Complex Systems Analyzer".to_string(),
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
    event_receiver: mpsc::Receiver<UIEvent>,
    callbacks: Vec<(UIEvent, Box<dyn Fn(UIEvent) + Send>)>,
}

impl UIBridge {
    pub fn new(config: UIConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();
        let state = Arc::new(AppState::new(config));
        
        Self {
            state,
            event_sender,
            event_receiver,
            callbacks: Vec::new(),
        }
    }

    pub fn initialize(&self) -> Result<()> {
        Ok(())
    }

    pub fn handle_command(&self, command: UICommand) -> Result<CommandResponse> {
        match command {
            UICommand::UpdateComponent(update) => {
                if let Some(action) = update.properties.get("action").and_then(|v| v.as_str()) {
                    match action {
                        "add_node" => self.event_sender.send(UIEvent::MenuAction(MenuAction::AddNode))
                            .map_err(|e| crate::error::Error::system(format!("Failed to send UI event: {}", e)))?,
                        "zoom_in" => self.event_sender.send(UIEvent::MenuAction(MenuAction::ZoomIn))
                            .map_err(|e| crate::error::Error::system(format!("Failed to send UI event: {}", e)))?,
                        "zoom_out" => self.event_sender.send(UIEvent::MenuAction(MenuAction::ZoomOut))
                            .map_err(|e| crate::error::Error::system(format!("Failed to send UI event: {}", e)))?,
                        "fit_to_view" => self.event_sender.send(UIEvent::MenuAction(MenuAction::FitToView))
                            .map_err(|e| crate::error::Error::system(format!("Failed to send UI event: {}", e)))?,
                        _ => {}
                    }
                }
                Ok(CommandResponse {
                    success: true,
                    data: None,
                    error: None,
                })
            },
            _ => self.state.handle_command(command)
        }
    }

    pub fn update_view(&self, update: ViewUpdate) -> Result<()> {
        // Process view updates
        for component in update.component_updates {
            self.handle_command(UICommand::UpdateComponent(component))?;
        }
        Ok(())
    }

    pub fn register_callback<F>(&mut self, event: UIEvent, callback: F) -> Result<()>
    where
        F: Fn(UIEvent) + Send + 'static,
    {
        self.callbacks.push((event, Box::new(callback)));
        Ok(())
    }

    pub fn process_events(&self) -> Result<()> {
        while let Ok(event) = self.event_receiver.try_recv() {
            for (ref registered_event, ref callback) in &self.callbacks {
                if std::mem::discriminant(registered_event) == std::mem::discriminant(&event) {
                    callback(event.clone());
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LayoutParams {
    pub spacing: f32,
    pub iterations: u32,
    pub force_strength: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    AddNode,
    AddEdge,
    DeleteSelected,
    ZoomIn,
    ZoomOut,
    FitToView,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UIEvent {
    GraphUpdated,
    SelectionChanged(Vec<String>),
    MenuAction(MenuAction),
    StatusUpdate(String),
    AnalysisCompleted(AnalysisResult),
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

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisResult {
    Centrality(Vec<(String, f64)>),
    Clustering(Vec<Vec<String>>),
    Paths(Vec<(String, String, Vec<String>)>),
} 