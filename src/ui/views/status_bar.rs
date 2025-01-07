use std::sync::Arc;
use crate::error::Result;
use crate::ui::{AppState, UIEvent};
use super::View;

pub struct StatusBarView {
    state: Arc<AppState>,
    status_text: String,
}

impl StatusBarView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            status_text: String::new(),
        }
    }
}

impl View for StatusBarView {
    fn initialize(&mut self) -> Result<()> {
        self.status_text = "Ready".to_string();
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_event(&mut self, event: &UIEvent) -> Result<()> {
        if let UIEvent::StatusUpdate(text) = event {
            self.status_text = text.clone();
        }
        Ok(())
    }
} 