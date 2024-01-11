pub trait FromBits<const BITS: usize> {
    fn from_bits(value: usize) -> Self;
}

impl FromBits<16> for u16 {
    fn from_bits(value: usize) -> Self {
        value as u16
    }
}

impl FromBits<8> for u8 {
    fn from_bits(value: usize) -> Self {
        value as u8
    }
}

impl FromBits<1> for bool {
    fn from_bits(value: usize) -> Self {
        value != 0
    }
}
