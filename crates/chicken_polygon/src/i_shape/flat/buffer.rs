use crate::i_float::int::point::IntPoint;
use crate::i_shape::int::shape::{IntContour, IntShape, IntShapes};
use alloc::vec::Vec;
use core::ops::Range;

#[derive(Debug, Clone, Default)]
pub struct FlatContoursBuffer {
    pub points: Vec<IntPoint>,
    pub ranges: Vec<Range<usize>>,
}

impl FlatContoursBuffer {
    pub fn store_contour(&mut self, contour: IntContour) {
        let start = self.points.len();
        self.points.extend(contour);
        let end = self.points.len();
        self.ranges.push(start..end);
    }

    pub fn store_shape(&mut self, shape: IntShape) {
        for contour in shape {
            self.store_contour(contour);
        }
    }

    pub fn store_shapes(&mut self, shapes: IntShapes) {
        for shape in shapes {
            self.store_shape(shape);
        }
    }

    pub fn clear_and_reserve(&mut self, count: usize, capacity: usize) {
        self.points.clear();
        self.ranges.clear();
        self.points.reserve(capacity); // Assuming capacity is for points
        self.ranges.reserve(count);
    }

    pub fn add_contour(&mut self, points: &[IntPoint]) {
        let start = self.points.len();
        self.points.extend_from_slice(points);
        let end = self.points.len();
        self.ranges.push(start..end);
    }

    pub fn is_single_contour(&self) -> bool {
        self.ranges.len() == 1
    }

    pub fn as_first_contour(&self) -> &[IntPoint] {
        if let Some(range) = self.ranges.first() {
            &self.points[range.clone()]
        } else {
            &[]
        }
    }

    pub fn as_first_contour_mut(&mut self) -> &mut [IntPoint] {
        if let Some(range) = self.ranges.first().cloned() {
            &mut self.points[range]
        } else {
            &mut []
        }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}
