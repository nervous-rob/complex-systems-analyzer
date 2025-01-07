use std::sync::Arc;
use crate::error::Result;
use super::{AppState, UIConfig, UIEvent, UICommand, CommandResponse, views::ViewManager};

pub struct App {
    state: Arc<AppState>,
    bridge: super::UIBridge,
    view_manager: ViewManager,
}

impl App {
    pub fn new(config: UIConfig) -> Result<Self> {
        let state = Arc::new(AppState::new(config.clone()));
        let (event_sender, event_receiver) = std::sync::mpsc::channel();
        
        let bridge = super::UIBridge {
            state: Arc::clone(&state),
            event_sender,
        };

        let view_manager = ViewManager::new(Arc::clone(&state));

        Ok(Self {
            state,
            bridge,
            view_manager,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        // Initialize views
        self.setup_views()?;
        
        // Initialize event handlers
        self.setup_event_handlers()?;
        
        // Initialize visualization
        let vis = self.state.get_visualization();
        vis.write()?.initialize()?;

        Ok(())
    }

    fn setup_views(&mut self) -> Result<()> {
        // Initialize the view manager
        self.view_manager.initialize()
    }

    fn setup_event_handlers(&self) -> Result<()> {
        // Register event handlers for UI events
        self.bridge.register_callback(UIEvent::GraphUpdated, Box::new(|event| {
            // Handle graph updates
            match event {
                UIEvent::GraphUpdated => {
                    // Update graph visualization
                }
                _ => {}
            }
        }))?;

        self.bridge.register_callback(UIEvent::SelectionChanged(vec![]), Box::new(|event| {
            // Handle selection changes
            match event {
                UIEvent::SelectionChanged(ids) => {
                    // Update selected components
                }
                _ => {}
            }
        }))?;

        Ok(())
    }

    pub fn handle_command(&self, command: UICommand) -> Result<CommandResponse> {
        self.bridge.handle_command(command)
    }

    pub fn update(&mut self) -> Result<()> {
        // Update views
        self.view_manager.update()?;
        
        // Update visualization
        let vis = self.state.get_visualization();
        vis.write()?.render_frame()?;

        Ok(())
    }
} 