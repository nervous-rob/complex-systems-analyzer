use std::fmt::Debug;
use super::{Point2D, Bounds2D, Spatial, SpatialIndex};

const MIN_ENTRIES: usize = 4;
const MAX_ENTRIES: usize = 8;

type NodeId = usize;

pub struct RTree<T: Spatial + Debug> {
    arena: Vec<Node<T>>,
    root: Option<NodeId>,
    size: usize,
}

#[derive(Clone)]
struct Node<T: Spatial + Debug> {
    bounds: Bounds2D,
    entries: Vec<Entry<T>>,
}

#[derive(Clone)]
enum Entry<T: Spatial + Debug> {
    Leaf(T),
    Node(NodeId),
}

impl<T: Spatial + Debug + Clone> RTree<T> {
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            root: None,
            size: 0,
        }
    }

    fn alloc_node(&mut self, node: Node<T>) -> NodeId {
        let id = self.arena.len();
        self.arena.push(node);
        id
    }

    fn choose_leaf(&mut self, item: &T) -> NodeId {
        if self.root.is_none() {
            let node = Node {
                bounds: item.bounds(),
                entries: Vec::new(),
            };
            let root_id = self.alloc_node(node);
            self.root = Some(root_id);
            return root_id;
        }

        let mut current_id = self.root.unwrap();
        loop {
            let current = &self.arena[current_id];
            if current.entries.is_empty() || matches!(current.entries[0], Entry::Leaf(_)) {
                break;
            }

            let mut min_idx = 0;
            let mut min_enlargement = f32::INFINITY;
            
            for (i, entry) in current.entries.iter().enumerate() {
                if let Entry::Node(child_id) = entry {
                    let child = &self.arena[*child_id];
                    let enlargement = Self::enlargement_needed(&child.bounds, &item.bounds());
                    if enlargement < min_enlargement {
                        min_enlargement = enlargement;
                        min_idx = i;
                    }
                }
            }

            match &current.entries[min_idx] {
                Entry::Node(next_id) => current_id = *next_id,
                _ => break,
            }
        }
        current_id
    }

    fn enlargement_needed(current: &Bounds2D, new_item: &Bounds2D) -> f32 {
        let new_bounds = Bounds2D::new(
            current.min_x.min(new_item.min_x),
            current.min_y.min(new_item.min_y),
            current.max_x.max(new_item.max_x),
            current.max_y.max(new_item.max_y),
        );

        let current_area = current.width() * current.height();
        let new_area = new_bounds.width() * new_bounds.height();
        new_area - current_area
    }

    fn adjust_bounds(&mut self, node_id: NodeId) {
        let bounds = {
            let node = &self.arena[node_id];
            if node.entries.is_empty() {
                return;
            }

            let mut bounds = match &node.entries[0] {
                Entry::Leaf(item) => item.bounds(),
                Entry::Node(child_id) => self.arena[*child_id].bounds,
            };

            for entry in node.entries.iter().skip(1) {
                match entry {
                    Entry::Leaf(item) => {
                        let item_bounds = item.bounds();
                        bounds = Bounds2D::new(
                            bounds.min_x.min(item_bounds.min_x),
                            bounds.min_y.min(item_bounds.min_y),
                            bounds.max_x.max(item_bounds.max_x),
                            bounds.max_y.max(item_bounds.max_y),
                        );
                    }
                    Entry::Node(child_id) => {
                        let child = &self.arena[*child_id];
                        bounds = Bounds2D::new(
                            bounds.min_x.min(child.bounds.min_x),
                            bounds.min_y.min(child.bounds.min_y),
                            bounds.max_x.max(child.bounds.max_x),
                            bounds.max_y.max(child.bounds.max_y),
                        );
                    }
                }
            }
            bounds
        };

        self.arena[node_id].bounds = bounds;
    }
}

impl<T: Spatial + Debug + Clone> SpatialIndex<T> for RTree<T> {
    fn insert(&mut self, item: T) {
        let leaf_id = self.choose_leaf(&item);
        self.arena[leaf_id].entries.push(Entry::Leaf(item));
        self.adjust_bounds(leaf_id);
        self.size += 1;
    }

    fn remove(&mut self, item: &T) -> Option<T> {
        let target_bounds = item.bounds();
        let mut removed_item = None;
        
        if let Some(root_id) = self.root {
            let mut stack = vec![root_id];

            while let Some(node_id) = stack.pop() {
                if !self.arena[node_id].bounds.intersects(&target_bounds) {
                    continue;
                }

                if let Some(idx) = self.arena[node_id].entries.iter().position(|entry| {
                    matches!(entry, Entry::Leaf(leaf) if leaf.bounds() == target_bounds)
                }) {
                    if let Entry::Leaf(item) = self.arena[node_id].entries.remove(idx) {
                        self.adjust_bounds(node_id);
                        removed_item = Some(item);
                        break;
                    }
                }

                let node = &self.arena[node_id];
                for entry in &node.entries {
                    if let Entry::Node(child_id) = entry {
                        stack.push(*child_id);
                    }
                }
            }
        }

        if removed_item.is_some() {
            self.size -= 1;
        }
        removed_item
    }

    fn query(&self, bounds: &Bounds2D) -> Vec<&T> {
        let mut result = Vec::new();
        if let Some(root_id) = self.root {
            let mut stack = vec![root_id];

            while let Some(node_id) = stack.pop() {
                let node = &self.arena[node_id];
                if node.bounds.intersects(bounds) {
                    for entry in &node.entries {
                        match entry {
                            Entry::Leaf(item) => {
                                if bounds.intersects(&item.bounds()) {
                                    result.push(item);
                                }
                            }
                            Entry::Node(child_id) => {
                                stack.push(*child_id);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    fn nearest(&self, point: Point2D, k: usize) -> Vec<&T> {
        let mut result = Vec::new();
        if let Some(root_id) = self.root {
            let mut candidates = Vec::new();
            let mut stack = vec![root_id];

            while let Some(node_id) = stack.pop() {
                let node = &self.arena[node_id];
                for entry in &node.entries {
                    match entry {
                        Entry::Leaf(item) => {
                            let dist = item.position().distance_to(&point);
                            candidates.push((dist, item));
                        }
                        Entry::Node(child_id) => {
                            stack.push(*child_id);
                        }
                    }
                }
            }

            candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            result.extend(candidates.iter().take(k).map(|(_, item)| item));
        }
        result
    }

    fn len(&self) -> usize {
        self.size
    }
} 