use alloc::vec::Vec;
use crate::i_float::int::point::IntPoint;

pub type IntContour = Vec<IntPoint>;
pub type IntShape = Vec<IntContour>;
pub type IntShapes = Vec<IntShape>;