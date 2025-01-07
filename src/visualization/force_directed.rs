use std::collections::HashMap;
use uuid::Uuid;

/// 2D point representation
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Force-directed layout calculator
pub struct ForceDirectedLayout {
    positions: HashMap<Uuid, Point>,
    velocities: HashMap<Uuid, Point>,
    repulsion: f32,
    attraction: f32,
    damping: f32,
}

impl ForceDirectedLayout {
    pub fn new(repulsion: f32, attraction: f32, damping: f32) -> Self {
        Self {
            positions: HashMap::new(),
            velocities: HashMap::new(),
            repulsion,
            attraction,
            damping,
        }
    }

    pub fn add_node(&mut self, id: Uuid, initial_pos: Option<Point>) {
        let pos = initial_pos.unwrap_or_else(|| Point::new(
            rand::random::<f32>() * 100.0,
            rand::random::<f32>() * 100.0,
        ));
        self.positions.insert(id, pos);
        self.velocities.insert(id, Point::new(0.0, 0.0));
    }

    pub fn step(&mut self, node_ids: &[Uuid], edges: &[(Uuid, Uuid)]) {
        // Calculate repulsive forces between all nodes
        for &id1 in node_ids {
            let mut force = Point::new(0.0, 0.0);
            let pos1 = self.positions[&id1];

            // Repulsion between nodes
            for &id2 in node_ids {
                if id1 != id2 {
                    let pos2 = self.positions[&id2];
                    let dist = pos1.distance(&pos2);
                    if dist > 0.0 {
                        let repulse = self.repulsion / (dist * dist);
                        force.x += repulse * (pos1.x - pos2.x) / dist;
                        force.y += repulse * (pos1.y - pos2.y) / dist;
                    }
                }
            }

            // Attraction along edges
            for &(source, target) in edges {
                if source == id1 || target == id1 {
                    let other_id = if source == id1 { target } else { source };
                    let pos2 = self.positions[&other_id];
                    let dist = pos1.distance(&pos2);
                    force.x -= self.attraction * (pos1.x - pos2.x);
                    force.y -= self.attraction * (pos1.y - pos2.y);
                }
            }

            // Update velocity and position
            let vel = self.velocities.get_mut(&id1).unwrap();
            vel.x = (vel.x + force.x) * self.damping;
            vel.y = (vel.y + force.y) * self.damping;

            let pos = self.positions.get_mut(&id1).unwrap();
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }

    pub fn get_position(&self, id: &Uuid) -> Option<Point> {
        self.positions.get(id).copied()
    }
} 