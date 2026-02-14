use core::cmp::Ordering;
use core::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct IntPoint {
    pub x: i32,
    pub y: i32,
}

impl IntPoint {
    pub const ZERO: IntPoint = IntPoint { x: 0, y: 0 };
    pub const EMPTY: IntPoint = IntPoint {
        x: i32::MAX,
        y: i32::MAX,
    };

    #[inline]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn subtract(&self, other: IntPoint) -> IntPoint {
        IntPoint {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    #[inline]
    pub fn add(&self, other: IntPoint) -> IntPoint {
        IntPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    #[inline]
    pub fn cross_product(&self, other: IntPoint) -> i64 {
        (self.x as i64 * other.y as i64) - (self.y as i64 * other.x as i64)
    }

    #[inline]
    pub fn dot_product(&self, other: IntPoint) -> i64 {
        (self.x as i64 * other.x as i64) + (self.y as i64 * other.y as i64)
    }

    #[inline]
    pub fn sqr_distance(&self, other: IntPoint) -> i64 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        dx * dx + dy * dy
    }

    #[inline]
    pub fn sqr_dist(&self, other: IntPoint) -> i64 {
        self.sqr_distance(other)
    }

    #[inline]
    pub fn sqr_length(&self) -> i64 {
        let x = self.x as i64;
        let y = self.y as i64;
        x * x + y * y
    }
}

impl Add for IntPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for IntPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.subtract(other)
    }
}
