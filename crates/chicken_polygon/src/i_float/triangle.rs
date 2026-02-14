use crate::i_float::int::point::IntPoint;
use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle {
    pub a: IntPoint,
    pub b: IntPoint,
    pub c: IntPoint,
}

impl Triangle {
    pub fn new(a: IntPoint, b: IntPoint, c: IntPoint) -> Self {
        Self { a, b, c }
    }

    // Cross product value: (b - a) x (c - a)
    #[inline]
    fn cross_product(a: IntPoint, b: IntPoint, c: IntPoint) -> i64 {
        (b.x as i64 - a.x as i64) * (c.y as i64 - a.y as i64)
            - (b.y as i64 - a.y as i64) * (c.x as i64 - a.x as i64)
    }

    pub fn is_clockwise_point(a: IntPoint, b: IntPoint, c: IntPoint) -> bool {
        Self::cross_product(a, b, c) < 0
    }

    pub fn clock_order_point(a: IntPoint, b: IntPoint, c: IntPoint) -> Ordering {
        let val = Self::cross_product(a, b, c);
        if val > 0 {
            Ordering::Less // CCW
        } else if val < 0 {
            Ordering::Greater // CW
        } else {
            Ordering::Equal // Collinear
        }
    }

    pub fn clock_direction_point(a: IntPoint, b: IntPoint, c: IntPoint) -> i32 {
        let val = Self::cross_product(a, b, c);
        if val > 0 {
            1
        } else if val < 0 {
            -1
        } else {
            0
        }
    }

    pub fn is_line_point(a: IntPoint, p: IntPoint, b: IntPoint) -> bool {
        Self::cross_product(a, p, b) == 0
    }

    pub fn is_not_line_point(a: IntPoint, p: IntPoint, b: IntPoint) -> bool {
        Self::cross_product(a, p, b) != 0
    }

    pub fn is_cw_or_line_point(a: IntPoint, b: IntPoint, c: IntPoint) -> bool {
        Self::cross_product(a, b, c) <= 0
    }

    // Signed area * 2 (which is the cross product)
    pub fn area_two_point(a: IntPoint, b: IntPoint, c: IntPoint) -> i64 {
        Self::cross_product(a, b, c)
    }

    pub fn is_contain_point_exclude_borders(
        p: IntPoint,
        a: IntPoint,
        b: IntPoint,
        c: IntPoint,
    ) -> bool {
        let cp1 = Self::cross_product(a, b, p);
        let cp2 = Self::cross_product(b, c, p);
        let cp3 = Self::cross_product(c, a, p);
        // If all have same sign (and non-zero), it's inside
        (cp1 > 0 && cp2 > 0 && cp3 > 0) || (cp1 < 0 && cp2 < 0 && cp3 < 0)
    }
}
