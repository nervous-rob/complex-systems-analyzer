use std::sync::Arc;
use crate::error::Result;
use crate::ui::{AppState, UIEvent, MenuAction, UICommand, ComponentUpdate};
use super::View;
use crate::ui::widgets::Button;
use serde_json::json;

pub struct ToolbarView {
    state: Arc<AppState>,
    buttons: Vec<Button>,
}

impl ToolbarView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            buttons: Vec::new(),
        }
    }

    fn create_button(&mut self, label: &str, action: &str) -> Result<()> {
        let mut btn = Button::new(label);
        let state = Arc::clone(&self.state);
        let action = action.to_string();
        
        btn.on_click(move || {
            state.handle_command(UICommand::UpdateComponent(ComponentUpdate {
                id: String::new(),
                properties: json!({
                    "type": "menu_action",
                    "action": action
                }),
            }))
            .map(|_| ())
        });
        
        self.buttons.push(btn);
        Ok(())
    }

    fn setup_buttons(&mut self) -> Result<()> {
        // Add Node button
        self.create_button("Add Node", "add_node")?;
        
        // Zoom controls
        self.create_button("Zoom In", "zoom_in")?;
        self.create_button("Zoom Out", "zoom_out")?;
        self.create_button("Fit to View", "fit_to_view")?;

        Ok(())
    }
}

impl View for ToolbarView {
    fn initialize(&mut self) -> Result<()> {
        self.setup_buttons()?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_event(&mut self, _event: &UIEvent) -> Result<()> {
        Ok(())
    }
} 