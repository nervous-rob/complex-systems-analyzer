use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Represents a node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Represents an edge in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub label: String,
    pub weight: f64,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Main graph data structure
#[derive(Debug)]
pub struct Graph {
    nodes: HashMap<Uuid, Node>,
    edges: HashMap<Uuid, Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge.id, edge);
    }

    pub fn get_node(&self, id: &Uuid) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_edge(&self, id: &Uuid) -> Option<&Edge> {
        self.edges.get(id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }
} 