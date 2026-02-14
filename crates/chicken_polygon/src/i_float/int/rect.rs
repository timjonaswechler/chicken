use super::point::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntRect {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl IntRect {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn with_point(p: IntPoint) -> Self {
        Self {
            min_x: p.x,
            min_y: p.y,
            max_x: p.x,
            max_y: p.y,
        }
    }

    pub fn is_intersect_border_include(&self, other: &IntRect) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn contains_with_radius(&self, p: IntPoint, radius: i32) -> bool {
        p.x >= self.min_x - radius
            && p.x <= self.max_x + radius
            && p.y >= self.min_y - radius
            && p.y <= self.max_y + radius
    }

    pub fn contains(&self, p: IntPoint) -> bool {
        p.x >= self.min_x && p.x <= self.max_x && p.y >= self.min_y && p.y <= self.max_y
    }

    pub fn unsafe_add_point(&mut self, p: IntPoint) {
        self.min_x = self.min_x.min(p.x);
        self.max_x = self.max_x.max(p.x);
        self.min_y = self.min_y.min(p.y);
        self.max_y = self.max_y.max(p.y);
    }
}
