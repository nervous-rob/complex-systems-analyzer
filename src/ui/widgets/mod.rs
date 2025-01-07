mod node;
mod edge;
mod controls;

pub use node::NodeWidget;
pub use edge::EdgeWidget;
pub use controls::*;

use std::sync::Arc;
use crate::error::Result;
use crate::ui::AppState;
use crate::core::SystemExt;

pub trait Widget {
    fn render(&self) -> Result<()>;
    fn handle_interaction(&self, event: WidgetEvent) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum WidgetEvent {
    Click,
    Hover,
    DragStart,
    DragEnd,
    DragMove { dx: f32, dy: f32 },
    Scroll { delta: f32 },
    KeyPress(String),
}

pub struct WidgetManager {
    state: Arc<AppState>,
    node_widgets: Vec<NodeWidget>,
    edge_widgets: Vec<EdgeWidget>,
}

impl WidgetManager {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            node_widgets: Vec::new(),
            edge_widgets: Vec::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.update_widgets()?;
        Ok(())
    }

    pub fn update_widgets(&mut self) -> Result<()> {
        let system = self.state.get_system();
        let system = system.read()?;
        
        // Update node widgets
        self.node_widgets = system.components()
            .iter()
            .map(|(_, component)| NodeWidget::new(component.clone()))
            .collect();
            
        // Update edge widgets
        self.edge_widgets = system.relationships()
            .iter()
            .map(|(_, relationship)| EdgeWidget::new(relationship.clone()))
            .collect();
            
        Ok(())
    }

    pub fn render_all(&self) -> Result<()> {
        for widget in &self.node_widgets {
            widget.render()?;
        }
        
        for widget in &self.edge_widgets {
            widget.render()?;
        }
        
        Ok(())
    }

    pub fn handle_interaction(&self, event: WidgetEvent) -> Result<()> {
        // Delegate interaction handling to appropriate widgets
        for widget in &self.node_widgets {
            widget.handle_interaction(event.clone())?;
        }
        
        for widget in &self.edge_widgets {
            widget.handle_interaction(event.clone())?;
        }
        
        Ok(())
    }
} 