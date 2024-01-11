pub trait FromBits<const BITS: usize> {
    fn from_bits(value: usize) -> Self;
}
