use crate::i_triangle::geom::point::IndexPoint;
use crate::i_tree::set::sort::KeyValue;
use crate::i_triangle::int::monotone::v_segment::VSegment;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub(super) struct FlatSection {
    pub(super) sort: VSegment,
    pub(super) points: Vec<IndexPoint>,
}

impl Default for FlatSection {
    #[inline]
    fn default() -> Self {
        Self {
            sort: Default::default(),
            points: Vec::new(),
        }
    }
}

impl KeyValue<VSegment> for FlatSection {
    #[inline]
    fn key(&self) -> &VSegment {
        &self.sort
    }
}
