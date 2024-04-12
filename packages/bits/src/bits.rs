use crate::bitrange;

pub trait Bits: Copy {
    fn slice(&self, most_significant_idx: u8, least_significant_idx: u8) -> impl Bits;

    fn full(&self) -> usize;
}

impl Bits for usize {
    fn slice(&self, most_significant_idx: u8, least_significant_idx: u8) -> Self {
        bitrange(most_significant_idx, least_significant_idx).of(*self)
    }

    fn full(&self) -> usize {
        *self
    }
}
