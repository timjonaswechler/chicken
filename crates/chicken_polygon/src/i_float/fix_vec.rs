use crate::i_float::int::point::IntPoint;
use core::ops::Sub;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FixVec {
    pub x: i64,
    pub y: i64,
}

impl FixVec {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn new_point(p: IntPoint) -> Self {
        Self {
            x: p.x as i64,
            y: p.y as i64,
        }
    }

    pub fn cross_product(&self, other: FixVec) -> i64 {
        self.x * other.y - self.y * other.x
    }

    pub fn dot_product(&self, other: FixVec) -> i64 {
        self.x * other.x + self.y * other.y
    }

    pub fn subtract(&self, other: FixVec) -> FixVec {
        FixVec {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub for FixVec {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.subtract(other)
    }
}
