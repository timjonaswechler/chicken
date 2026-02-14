#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UInt128 {
    pub low: u64,
    pub high: u64,
}

impl UInt128 {
    pub fn multiply(a: u64, b: u64) -> Self {
        let res = a as u128 * b as u128;
        Self {
            low: res as u64,
            high: (res >> 64) as u64,
        }
    }
}
