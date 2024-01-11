pub trait FromBits<const BITS: usize> {
    fn from_bits(value: usize) -> Self;
}

impl FromBits<16> for u16 {
    fn from_bits(value: usize) -> Self {
        value as u16
    }
}
