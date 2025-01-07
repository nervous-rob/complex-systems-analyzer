use std::sync::Arc;
use crate::error::Result;
use super::View;
use crate::ui::{AppState, UIEvent};

pub struct GraphView {
    state: Arc<AppState>,
}

impl GraphView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn handle_graph_update(&mut self) -> Result<()> {
        let vis = self.state.get_visualization();
        let system = self.state.get_system();
        
        let mut vis = vis.write()?;
        let system = system.read()?;
        
        vis.update_graph(&system)?;
        vis.render()?;
        
        Ok(())
    }

    fn handle_selection(&mut self, selected_ids: &[String]) -> Result<()> {
        self.state.update_selection(selected_ids.to_vec())?;
        
        let vis = self.state.get_visualization();
        let mut vis = vis.write()?;
        
        // Update visualization to highlight selected components
        vis.update_selection(&selected_ids)?;
        vis.render()?;
        
        Ok(())
    }
}

impl View for GraphView {
    fn initialize(&mut self) -> Result<()> {
        // GraphView initialization is now handled by App
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        // Update visualization
        let vis = self.state.get_visualization();
        let mut vis = vis.write()?;
        vis.render()?;
        Ok(())
    }

    fn handle_event(&mut self, event: &UIEvent) -> Result<()> {
        match event {
            UIEvent::GraphUpdated => self.handle_graph_update()?,
            UIEvent::SelectionChanged(ids) => self.handle_selection(ids)?,
            _ => {}
        }
        Ok(())
    }
} 