use std::sync::Arc;
use crate::error::Result;
use super::View;
use crate::ui::{AppState, UIEvent, AnalysisConfig, AnalysisResult};
use crate::ui::widgets::{Button, Slider, Checkbox};

pub struct AnalysisView {
    state: Arc<AppState>,
    config_panel: ConfigPanel,
    results_panel: ResultsPanel,
}

struct ConfigPanel {
    title_button: Button,
    algorithm_options: Vec<Checkbox>,
    parameter_sliders: Vec<Slider>,
    run_button: Button,
    is_expanded: bool,
}

struct ResultsPanel {
    title_button: Button,
    result_text: String,
    export_button: Button,
    clear_button: Button,
    is_expanded: bool,
}

impl AnalysisView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state: Arc::clone(&state),
            config_panel: ConfigPanel {
                title_button: Button::new("Analysis Configuration"),
                algorithm_options: Vec::new(),
                parameter_sliders: Vec::new(),
                run_button: Button::new("Run Analysis"),
                is_expanded: true,
            },
            results_panel: ResultsPanel {
                title_button: Button::new("Analysis Results"),
                result_text: String::new(),
                export_button: Button::new("Export Results"),
                clear_button: Button::new("Clear Results"),
                is_expanded: true,
            },
        }
    }

    fn setup_config_panel(&mut self) -> Result<()> {
        // Set up algorithm options
        self.config_panel.algorithm_options = vec![
            Checkbox::new("Include Centrality Analysis"),
            Checkbox::new("Include Clustering Analysis"),
            Checkbox::new("Include Path Analysis"),
        ];

        // Set up parameter sliders
        self.config_panel.parameter_sliders = vec![
            Slider::new(0.0, 1.0), // Damping factor
            Slider::new(1.0, 100.0), // Max iterations
            Slider::new(0.0, 1.0), // Convergence threshold
        ];

        self.setup_run_button()?;
        Ok(())
    }

    fn setup_run_button(&mut self) -> Result<()> {
        let state = Arc::clone(&self.state);
        let button = &mut self.config_panel.run_button;
        
        let options = self.config_panel.algorithm_options.iter()
            .map(|c| c.is_checked())
            .collect::<Vec<_>>();
        let params = self.config_panel.parameter_sliders.iter()
            .map(|s| s.value())
            .collect::<Vec<_>>();
        
        button.on_click(move || {
            let config = AnalysisConfig {
                include_centrality: options[0],
                include_clustering: options[1],
                include_paths: options[2],
                damping_factor: params[0],
                max_iterations: params[1] as u32,
                convergence_threshold: params[2],
            };
            state.run_analysis(config)?;
            Ok(())
        });
        Ok(())
    }

    fn setup_results_panel(&mut self) -> Result<()> {
        // Set up export button
        let state = Arc::clone(&self.state);
        self.results_panel.export_button.on_click(move || {
            state.export_analysis_results("analysis_results.json")?;
            Ok(())
        });

        // Set up clear button
        let state = Arc::clone(&self.state);
        self.results_panel.clear_button.on_click(move || {
            state.clear_analysis_results()?;
            Ok(())
        });

        Ok(())
    }

    fn build_analysis_config(&self) -> Result<AnalysisConfig> {
        let mut config = AnalysisConfig {
            include_centrality: false,
            include_clustering: false,
            include_paths: false,
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_threshold: 0.001,
        };

        // Update config based on selected options and parameters
        for (i, checkbox) in self.config_panel.algorithm_options.iter().enumerate() {
            if checkbox.is_checked() {
                match i {
                    0 => config.include_centrality = true,
                    1 => config.include_clustering = true,
                    2 => config.include_paths = true,
                    _ => {}
                }
            }
        }

        // Update parameters from sliders
        if let Some(slider) = self.config_panel.parameter_sliders.get(0) {
            config.damping_factor = slider.value();
        }
        if let Some(slider) = self.config_panel.parameter_sliders.get(1) {
            config.max_iterations = slider.value() as u32;
        }
        if let Some(slider) = self.config_panel.parameter_sliders.get(2) {
            config.convergence_threshold = slider.value();
        }

        Ok(config)
    }

    fn update_results_display(&mut self, results: &AnalysisResult) -> Result<()> {
        self.results_panel.result_text.clear();
        
        // Format and display results
        match results {
            AnalysisResult::Centrality(scores) => {
                self.results_panel.result_text.push_str("Centrality Analysis Results:\n");
                for (node, score) in scores {
                    self.results_panel.result_text.push_str(
                        &format!("{}: {:.4}\n", node, score)
                    );
                }
            }
            AnalysisResult::Clustering(clusters) => {
                self.results_panel.result_text.push_str("Clustering Analysis Results:\n");
                for (i, cluster) in clusters.iter().enumerate() {
                    self.results_panel.result_text.push_str(
                        &format!("Cluster {}: {} nodes\n", i + 1, cluster.len())
                    );
                }
            }
            AnalysisResult::Paths(paths) => {
                self.results_panel.result_text.push_str("Path Analysis Results:\n");
                for (start, end, path) in paths {
                    self.results_panel.result_text.push_str(
                        &format!("{} -> {}: {} steps\n", start, end, path.len())
                    );
                }
            }
        }

        Ok(())
    }
}

impl View for AnalysisView {
    fn initialize(&mut self) -> Result<()> {
        // Set up panels
        self.setup_config_panel()?;
        self.setup_results_panel()?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        // Update panel states based on current state
        Ok(())
    }

    fn handle_event(&mut self, event: &UIEvent) -> Result<()> {
        match event {
            UIEvent::AnalysisCompleted(result) => {
                self.update_results_display(result)?;
            }
            _ => {}
        }
        Ok(())
    }
} 