mod graph;
mod sidebar;
mod toolbar;
mod analysis;

pub use graph::GraphView;
pub use sidebar::SidebarView;
pub use toolbar::ToolbarView;
pub use analysis::AnalysisView;

use crate::error::Result;
use std::sync::Arc;
use super::{AppState, UIEvent};

pub trait View {
    fn initialize(&mut self) -> Result<()>;
    fn update(&mut self) -> Result<()>;
    fn handle_event(&mut self, event: &UIEvent) -> Result<()>;
}

pub struct ViewManager {
    state: Arc<AppState>,
    graph_view: GraphView,
    sidebar_view: SidebarView,
    toolbar_view: ToolbarView,
    analysis_view: AnalysisView,
}

impl ViewManager {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            graph_view: GraphView::new(Arc::clone(&state)),
            sidebar_view: SidebarView::new(Arc::clone(&state)),
            toolbar_view: ToolbarView::new(Arc::clone(&state)),
            analysis_view: AnalysisView::new(Arc::clone(&state)),
            state,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.graph_view.initialize()?;
        self.sidebar_view.initialize()?;
        self.toolbar_view.initialize()?;
        self.analysis_view.initialize()?;
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        self.graph_view.update()?;
        self.sidebar_view.update()?;
        self.toolbar_view.update()?;
        self.analysis_view.update()?;
        Ok(())
    }

    pub fn handle_event(&mut self, event: &super::UIEvent) -> Result<()> {
        self.graph_view.handle_event(event)?;
        self.sidebar_view.handle_event(event)?;
        self.toolbar_view.handle_event(event)?;
        self.analysis_view.handle_event(event)?;
        Ok(())
    }
} 