use crate::i_float::fix_vec::FixVec;
use crate::i_float::int::point::IntPoint;
use core::cmp::Ordering;

#[derive(PartialEq, Eq)]
pub(super) enum ABCExcludeResult {
    Inside,
    Outside,
    OutsideEdge,
}

// a, b, c - counter clock wised points
pub(super) struct Abc {
    a: IntPoint,
    b: IntPoint,
    c: IntPoint,
    ab: FixVec,
    bc: FixVec,
    ca: FixVec,
}

impl Abc {
    #[inline(always)]
    pub(super) fn new(a: IntPoint, b: IntPoint, c: IntPoint) -> Self {
        let ab = FixVec::new_point(b.subtract(a));
        let bc = FixVec::new_point(c.subtract(b));
        let ca = FixVec::new_point(a.subtract(c));
        Self {
            a,
            b,
            c,
            ab,
            bc,
            ca,
        }
    }

    #[inline(always)]
    pub(super) fn contains(&self, p: IntPoint) -> bool {
        let ap = FixVec::new_point(p.subtract(self.a));
        let a_cross = ap.cross_product(self.ab);
        if a_cross >= 0 {
            return false;
        }

        let bp = FixVec::new_point(p.subtract(self.b));
        let b_cross = bp.cross_product(self.bc);
        if b_cross >= 0 {
            return false;
        }

        let cp = FixVec::new_point(p.subtract(self.c));
        let c_cross = cp.cross_product(self.ca);

        c_cross < 0
    }

    #[inline(always)]
    pub(super) fn contains_exclude_ca(&self, p: IntPoint) -> ABCExcludeResult {
        let ap = FixVec::new_point(p.subtract(self.a));
        let a_cross = ap.cross_product(self.ab);
        if a_cross >= 0 {
            return ABCExcludeResult::Outside;
        }

        let bp = FixVec::new_point(p.subtract(self.b));
        let b_cross = bp.cross_product(self.bc);
        if b_cross >= 0 {
            return ABCExcludeResult::Outside;
        }

        let cp = FixVec::new_point(p.subtract(self.c));
        let c_cross = cp.cross_product(self.ca);

        match c_cross.cmp(&0) {
            Ordering::Less => ABCExcludeResult::Inside,
            Ordering::Equal => {
                if AB::contains(self.a, self.c, p) {
                    ABCExcludeResult::Inside
                } else {
                    ABCExcludeResult::OutsideEdge
                }
            }
            Ordering::Greater => ABCExcludeResult::OutsideEdge,
        }
    }
}

pub(super) struct AB;

impl AB {
    #[inline(always)]
    pub(super) fn contains(a: IntPoint, b: IntPoint, p: IntPoint) -> bool {
        // a, b, p already on one line
        // not including ends
        let ap = FixVec::new_point(a.subtract(p));
        let bp = FixVec::new_point(b.subtract(p));

        // must have opposite direction
        ap.dot_product(bp) < 0
    }
}
