use std::fmt::Debug;
use super::{Point2D, Bounds2D, Spatial, SpatialIndex};

const MAX_ITEMS: usize = 8;
const MAX_DEPTH: u32 = 8;

pub struct QuadTree<T: Spatial + Debug> {
    root: QuadNode<T>,
    size: usize,
}

enum QuadNode<T: Spatial + Debug> {
    Leaf {
        bounds: Bounds2D,
        items: Vec<T>,
    },
    Internal {
        bounds: Bounds2D,
        children: Box<[QuadNode<T>; 4]>,
    },
}

impl<T: Spatial + Debug> QuadTree<T> {
    pub fn new(bounds: Bounds2D) -> Self {
        Self {
            root: QuadNode::Leaf {
                bounds,
                items: Vec::new(),
            },
            size: 0,
        }
    }

    fn split_node(bounds: &Bounds2D) -> [Bounds2D; 4] {
        let center = bounds.center();
        [
            // Northwest
            Bounds2D::new(bounds.min_x, center.y, center.x, bounds.max_y),
            // Northeast
            Bounds2D::new(center.x, center.y, bounds.max_x, bounds.max_y),
            // Southwest
            Bounds2D::new(bounds.min_x, bounds.min_y, center.x, center.y),
            // Southeast
            Bounds2D::new(center.x, bounds.min_y, bounds.max_x, center.y),
        ]
    }

    fn get_target_child(children: &[QuadNode<T>; 4], pos: &Point2D) -> usize {
        for (i, child) in children.iter().enumerate() {
            if let QuadNode::Leaf { bounds, .. } = child {
                if bounds.contains_point(pos) {
                    return i;
                }
            }
        }
        0 // Default to first quadrant if point doesn't fit exactly
    }

    fn split_leaf(bounds: &Bounds2D, items: Vec<T>) -> Box<[QuadNode<T>; 4]> {
        let child_bounds = Self::split_node(bounds);
        let mut children = Box::new([
            QuadNode::Leaf {
                bounds: child_bounds[0],
                items: Vec::new(),
            },
            QuadNode::Leaf {
                bounds: child_bounds[1],
                items: Vec::new(),
            },
            QuadNode::Leaf {
                bounds: child_bounds[2],
                items: Vec::new(),
            },
            QuadNode::Leaf {
                bounds: child_bounds[3],
                items: Vec::new(),
            },
        ]);

        // Redistribute existing items
        for item in items {
            let pos = item.position();
            let idx = Self::get_target_child(&children, &pos);
            if let QuadNode::Leaf { items, .. } = &mut children[idx] {
                items.push(item);
            }
        }

        children
    }

    fn insert_recursive(node: &mut QuadNode<T>, item: T, depth: u32) {
        match node {
            QuadNode::Leaf { bounds, items } => {
                if items.len() < MAX_ITEMS || depth >= MAX_DEPTH {
                    items.push(item);
                    return;
                }

                // Split node
                let old_bounds = *bounds;
                let mut old_items = Vec::new();
                std::mem::swap(items, &mut old_items);
                old_items.push(item);

                let children = Self::split_leaf(&old_bounds, old_items);
                *node = QuadNode::Internal {
                    bounds: old_bounds,
                    children,
                };
            }
            QuadNode::Internal { children, .. } => {
                let pos = item.position();
                let idx = Self::get_target_child(children, &pos);
                Self::insert_recursive(&mut children[idx], item, depth + 1);
            }
        }
    }
}

impl<T: Spatial + Debug> SpatialIndex<T> for QuadTree<T> {
    fn insert(&mut self, item: T) {
        Self::insert_recursive(&mut self.root, item, 0);
        self.size += 1;
    }

    fn remove(&mut self, item: &T) -> Option<T> {
        let pos = item.position();
        let mut removed_item = None;

        match &mut self.root {
            QuadNode::Leaf { items, .. } => {
                if let Some(index) = items.iter().position(|i| i.position() == pos) {
                    removed_item = Some(items.remove(index));
                }
            }
            QuadNode::Internal { children, .. } => {
                for child in children.iter_mut() {
                    if let QuadNode::Leaf { bounds, items } = child {
                        if bounds.contains_point(&pos) {
                            if let Some(index) = items.iter().position(|i| i.position() == pos) {
                                removed_item = Some(items.remove(index));
                                break;
                            }
                        }
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
        let mut stack = vec![&self.root];

        while let Some(node) = stack.pop() {
            match node {
                QuadNode::Leaf { bounds: node_bounds, items } => {
                    if bounds.intersects(node_bounds) {
                        for item in items {
                            if bounds.contains_point(&item.position()) {
                                result.push(item);
                            }
                        }
                    }
                }
                QuadNode::Internal { bounds: node_bounds, children } => {
                    if bounds.intersects(node_bounds) {
                        stack.extend(children.iter());
                    }
                }
            }
        }

        result
    }

    fn nearest(&self, point: Point2D, k: usize) -> Vec<&T> {
        let mut result = Vec::new();
        let mut stack = vec![&self.root];
        let mut nearest = Vec::new();

        while let Some(node) = stack.pop() {
            match node {
                QuadNode::Leaf { items, .. } => {
                    for item in items {
                        let dist = item.position().distance_to(&point);
                        nearest.push((dist, item));
                    }
                }
                QuadNode::Internal { children, .. } => {
                    stack.extend(children.iter());
                }
            }
        }

        nearest.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        result.extend(nearest.iter().take(k).map(|(_, item)| item));
        result
    }

    fn len(&self) -> usize {
        self.size
    }
} 