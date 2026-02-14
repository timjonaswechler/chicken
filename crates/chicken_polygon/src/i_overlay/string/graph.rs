use crate::i_overlay::build::builder::GraphNode;
use crate::i_overlay::core::link::OverlayLink;
use alloc::vec::Vec;

pub struct StringGraph<'a> {
    pub(crate) nodes: &'a [Vec<usize>],
    pub(crate) links: &'a mut [OverlayLink],
}

impl GraphNode for Vec<usize> {
    #[inline(always)]
    fn with_indices(indices: &[usize]) -> Self {
        indices.to_vec()
    }
}
