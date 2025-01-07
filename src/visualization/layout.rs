use std::collections::HashMap;
use uuid::Uuid;
use super::force_directed::{Point, ForceDirectedLayout};

/// Available layout algorithms
#[derive(Debug, Clone, Copy)]
pub enum LayoutAlgorithm {
    ForceDirected,
    Circular,
    Grid,
}

/// Layout manager that handles different layout strategies
pub struct LayoutManager {
    algorithm: LayoutAlgorithm,
    force_directed: Option<ForceDirectedLayout>,
    positions: HashMap<Uuid, Point>,
}

impl LayoutManager {
    pub fn new(algorithm: LayoutAlgorithm) -> Self {
        Self {
            algorithm,
            force_directed: None,
            positions: HashMap::new(),
        }
    }

    pub fn initialize_force_directed(&mut self) {
        self.force_directed = Some(ForceDirectedLayout::new(
            1.0,  // repulsion
            0.01, // attraction
            0.9,  // damping
        ));
    }

    pub fn layout_circular(&mut self, node_ids: &[Uuid]) {
        let node_count = node_ids.len() as f32;
        let radius = 100.0;
        let center = Point::new(0.0, 0.0);

        for (i, &id) in node_ids.iter().enumerate() {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / node_count;
            let pos = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            self.positions.insert(id, pos);
        }
    }

    pub fn layout_grid(&mut self, node_ids: &[Uuid]) {
        let node_count = node_ids.len() as f32;
        let cols = (node_count.sqrt().ceil()) as i32;
        let spacing = 50.0;

        for (i, &id) in node_ids.iter().enumerate() {
            let row = (i as i32) / cols;
            let col = (i as i32) % cols;
            let pos = Point::new(
                col as f32 * spacing,
                row as f32 * spacing,
            );
            self.positions.insert(id, pos);
        }
    }

    pub fn step(&mut self, node_ids: &[Uuid], edges: &[(Uuid, Uuid)]) {
        match self.algorithm {
            LayoutAlgorithm::ForceDirected => {
                if let Some(layout) = &mut self.force_directed {
                    // Ensure all nodes are initialized
                    for &id in node_ids {
                        if !layout.get_position(&id).is_some() {
                            layout.add_node(id, None);
                        }
                    }
                    layout.step(node_ids, edges);
                }
            }
            LayoutAlgorithm::Circular => self.layout_circular(node_ids),
            LayoutAlgorithm::Grid => self.layout_grid(node_ids),
        }
    }

    pub fn get_position(&self, id: &Uuid) -> Option<Point> {
        match self.algorithm {
            LayoutAlgorithm::ForceDirected => {
                self.force_directed.as_ref().and_then(|l| l.get_position(id))
            }
            _ => self.positions.get(id).copied(),
        }
    }
} 