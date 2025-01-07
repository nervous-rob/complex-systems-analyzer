pub mod renderer;

use std::sync::Arc;
use winit::window::Window;
use crate::error::Result;
use crate::core::System;
use uuid::Uuid;
use std::collections::HashMap;

pub struct Visualization {
    pub(crate) renderer: Option<Renderer>,
    initialized: bool,
}

impl Visualization {
    pub fn new() -> Self {
        Self {
            renderer: None,
            initialized: false,
        }
    }

    pub fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        println!("Starting visualization initialization...");
        let mut renderer = match pollster::block_on(Renderer::new(window)) {
            Ok(r) => {
                println!("Successfully created renderer");
                r
            },
            Err(e) => {
                println!("Failed to create renderer: {:?}", e);
                return Err(e);
            }
        };
        
        println!("Adding test nodes...");
        // Add some test nodes to verify rendering
        let test_nodes = vec![
            NodeData {
                id: Uuid::new_v4(),
                position: [-200.0, 0.0],
                size: 50.0,
                color: [1.0, 0.0, 0.0, 1.0],
                label: "Test Node 1".to_string(),
            },
            NodeData {
                id: Uuid::new_v4(),
                position: [200.0, 0.0],
                size: 50.0,
                color: [0.0, 1.0, 0.0, 1.0],
                label: "Test Node 2".to_string(),
            },
        ];
        
        if let Err(e) = renderer.update_graph_data(test_nodes, vec![]) {
            println!("Failed to update graph data: {:?}", e);
            return Err(e);
        }
        println!("Successfully updated graph data");

        if let Err(e) = renderer.fit_to_view() {
            println!("Failed to fit to view: {:?}", e);
            return Err(e);
        }
        println!("Successfully fit to view");
        
        self.renderer = Some(renderer);
        self.initialized = true;
        println!("Visualization initialization complete");
        Ok(())
    }

    pub fn update_graph(&mut self, system: &System) -> Result<()> {
        if let Some(renderer) = &mut self.renderer {
            // Convert system data to renderer nodes/edges
            let nodes = system.components.iter().map(|(id, comp)| NodeData {
                id: *id,
                position: [0.0, 0.0], // Initial position, will be adjusted by layout
                size: 50.0,
                color: [1.0, 0.0, 0.0, 1.0],
                label: comp.name.clone(),
            }).collect();

            renderer.update_graph_data(nodes, vec![])?;
            renderer.fit_to_view()?;
        }
        Ok(())
    }

    pub fn fit_to_view(&mut self) -> Result<()> {
        if let Some(renderer) = &mut self.renderer {
            renderer.fit_to_view()?;
        }
        Ok(())
    }

    pub fn update_selection(&mut self, selected_ids: &[String]) -> Result<()> {
        if let Some(renderer) = &mut self.renderer {
            let ids: Vec<Uuid> = selected_ids.iter()
                .filter_map(|id| Uuid::parse_str(id).ok())
                .collect();
            renderer.update_selection(&ids)?;
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        if let Some(renderer) = &mut self.renderer {
            renderer.render()?;
        }
        Ok(())
    }

    pub fn take_renderer(&mut self) -> Option<Renderer> {
        self.renderer.take()
    }
}

use self::renderer::{Renderer, NodeData, EdgeData}; 