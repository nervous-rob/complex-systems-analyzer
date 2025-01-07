use std::sync::Arc;
use crate::error::Result;
use super::View;
use crate::ui::{AppState, UIEvent, UICommand, LayoutType, AnalysisConfig};
use crate::ui::widgets::Button;

pub struct ToolbarView {
    state: Arc<AppState>,
    layout_buttons: Vec<Button>,
    analysis_buttons: Vec<Button>,
    export_button: Button,
    import_button: Button,
    zoom_buttons: Vec<Button>,
}

impl ToolbarView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state: Arc::clone(&state),
            layout_buttons: vec![
                Button::new("Force Directed"),
                Button::new("Hierarchical"),
                Button::new("Circular"),
            ],
            analysis_buttons: vec![
                Button::new("Centrality"),
                Button::new("Clustering"),
                Button::new("Path Analysis"),
            ],
            export_button: Button::new("Export"),
            import_button: Button::new("Import"),
            zoom_buttons: vec![
                Button::new("Zoom In"),
                Button::new("Zoom Out"),
                Button::new("Fit View"),
            ],
        }
    }

    fn setup_layout_buttons(&mut self) -> Result<()> {
        for button in &mut self.layout_buttons {
            let state = Arc::clone(&self.state);
            let label = button.label().to_string();
            
            button.on_click(move || {
                let layout_type = match label.as_str() {
                    "Force Directed" => LayoutType::ForceDirected,
                    "Hierarchical" => LayoutType::Hierarchical,
                    "Circular" => LayoutType::Circular,
                    _ => return Ok(()),
                };

                let mut config = state.get_ui_config()?;
                config.layout.layout_type = layout_type;
                state.update_config(config)?;

                let vis = state.get_visualization();
                vis.write()?.update_layout()?;
                Ok(())
            });
        }
        Ok(())
    }

    fn setup_analysis_buttons(&mut self) -> Result<()> {
        for button in &mut self.analysis_buttons {
            let state = Arc::clone(&self.state);
            let label = button.label().to_string();
            
            button.on_click(move || {
                let config = AnalysisConfig {
                    include_centrality: label == "Centrality",
                    include_clustering: label == "Clustering",
                    include_paths: label == "Path Analysis",
                    damping_factor: 0.85,
                    max_iterations: 100,
                    convergence_threshold: 0.001,
                };

                state.handle_command(UICommand::RunAnalysis(config))?;
                Ok(())
            });
        }
        Ok(())
    }

    fn setup_io_buttons(&mut self) -> Result<()> {
        let state = Arc::clone(&self.state);
        self.export_button.on_click(move || {
            state.handle_command(UICommand::ExportGraph("graph.json".to_string()))?;
            Ok(())
        });

        let state = Arc::clone(&self.state);
        self.import_button.on_click(move || {
            state.handle_command(UICommand::ImportGraph("graph.json".to_string()))?;
            Ok(())
        });

        Ok(())
    }

    fn setup_zoom_buttons(&mut self) -> Result<()> {
        for button in &mut self.zoom_buttons {
            let state = Arc::clone(&self.state);
            let label = button.label().to_string();
            
            button.on_click(move || {
                let vis = state.get_visualization();
                let mut vis = vis.write()?;

                match label.as_str() {
                    "Zoom In" => vis.zoom_in()?,
                    "Zoom Out" => vis.zoom_out()?,
                    "Fit View" => vis.fit_view()?,
                    _ => {}
                }
                Ok(())
            });
        }
        Ok(())
    }
}

impl View for ToolbarView {
    fn initialize(&mut self) -> Result<()> {
        self.setup_layout_buttons()?;
        self.setup_analysis_buttons()?;
        self.setup_io_buttons()?;
        self.setup_zoom_buttons()?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        // Update button states based on current system state
        let system = self.state.get_system();
        let system = system.read()?;

        // Enable/disable analysis buttons based on system state
        let has_data = !system.is_empty();
        for button in &mut self.analysis_buttons {
            button.set_enabled(has_data);
        }

        // Enable/disable export button
        self.export_button.set_enabled(has_data);

        Ok(())
    }

    fn handle_event(&mut self, event: &UIEvent) -> Result<()> {
        match event {
            UIEvent::GraphUpdated => {
                self.update()?;
            }
            UIEvent::AnalysisStarted => {
                // Disable analysis buttons during analysis
                for button in &mut self.analysis_buttons {
                    button.set_enabled(false);
                }
            }
            UIEvent::AnalysisCompleted => {
                // Re-enable analysis buttons after completion
                self.update()?;
            }
            _ => {} // Ignore other events
        }
        Ok(())
    }
} 