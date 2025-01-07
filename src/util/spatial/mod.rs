pub mod quadtree;
pub mod rtree;

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds2D {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Trait for objects that can be spatially indexed
pub trait Spatial {
    fn bounds(&self) -> Bounds2D;
    fn position(&self) -> Point2D;
}

/// Common trait for spatial indexing structures
pub trait SpatialIndex<T: Spatial + Debug> {
    /// Insert an item into the index
    fn insert(&mut self, item: T);
    
    /// Remove an item from the index
    fn remove(&mut self, item: &T) -> Option<T>;
    
    /// Query items within the given bounds
    fn query(&self, bounds: &Bounds2D) -> Vec<&T>;
    
    /// Find nearest neighbors to a point
    fn nearest(&self, point: Point2D, k: usize) -> Vec<&T>;
    
    /// Get the total number of items in the index
    fn len(&self) -> usize;
    
    /// Check if the index is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Bounds2D {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_points(points: &[Point2D]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut bounds = Self {
            min_x: points[0].x,
            min_y: points[0].y,
            max_x: points[0].x,
            max_y: points[0].y,
        };

        for point in points.iter().skip(1) {
            bounds.min_x = bounds.min_x.min(point.x);
            bounds.min_y = bounds.min_y.min(point.y);
            bounds.max_x = bounds.max_x.max(point.x);
            bounds.max_y = bounds.max_y.max(point.y);
        }

        Some(bounds)
    }

    pub fn contains_point(&self, point: &Point2D) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    pub fn intersects(&self, other: &Bounds2D) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn center(&self) -> Point2D {
        Point2D {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
        }
    }
} 