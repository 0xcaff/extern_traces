use crate::bitrange;

pub trait Bits: Copy {
    fn slice(&self, most_significant_idx: u8, least_significant_idx: u8) -> impl Bits;

    fn full(&self) -> u64;
}

impl Bits for u64 {
    fn slice(&self, most_significant_idx: u8, least_significant_idx: u8) -> Self {
        debug_assert!(most_significant_idx < 64);
        debug_assert!(least_significant_idx < 64);
        debug_assert!(most_significant_idx >= least_significant_idx);

        bitrange(most_significant_idx, least_significant_idx).of(*self)
    }

    fn full(&self) -> u64 {
        *self
    }
}

impl Bits for u32 {
    fn slice(&self, most_significant_idx: u8, least_significant_idx: u8) -> impl Bits {
        debug_assert!(most_significant_idx < 32);
        debug_assert!(least_significant_idx < 32);
        debug_assert!(most_significant_idx >= least_significant_idx);

        (*self as u64).slice(most_significant_idx, least_significant_idx)
    }

    fn full(&self) -> u64 {
        *self as _
    }
}

impl Bits for &[u64] {
    fn slice(&self, most_significant_idx: u8, least_significant_idx: u8) -> impl Bits {
        let most_significant_elem_idx = most_significant_idx / 64;
        let least_significant_elem_idx = least_significant_idx / 64;

        assert_eq!(most_significant_elem_idx, least_significant_elem_idx);

        self[most_significant_elem_idx as usize]
            .slice(most_significant_idx % 64, least_significant_idx % 64)
    }

    fn full(&self) -> u64 {
        unimplemented!()
    }
}
