use crate::i_float::int::point::IntPoint;
use crate::i_triangle::advanced::delaunay::IntDelaunay;
use crate::i_triangle::int::triangulation::{IndexType, IntTriangulation};
use alloc::vec::Vec;

impl IntDelaunay {
    #[inline]
    pub fn points(&self) -> &Vec<IntPoint> {
        &self.points
    }

    #[inline]
    pub fn triangle_indices<I: IndexType>(&self) -> Vec<I> {
        let mut result = Vec::with_capacity(3 * self.triangles.len());
        for t in &self.triangles {
            let v = &t.vertices;
            let i0 = I::try_from(v[0].index).unwrap_or(I::ZERO);
            let i1 = I::try_from(v[1].index).unwrap_or(I::ZERO);
            let i2 = I::try_from(v[2].index).unwrap_or(I::ZERO);

            result.extend_from_slice(&[i0, i1, i2]);
        }
        result
    }

    #[inline]
    pub fn into_triangulation<I: IndexType>(self) -> IntTriangulation<I> {
        IntTriangulation {
            indices: self.triangle_indices(),
            points: self.points,
        }
    }
}
