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

impl FromBits<32> for u32 {
    fn from_bits(value: usize) -> Self {
        value as _
    }
}

macro_rules! from_bits_impls {
    ($($n:expr),*) => {
        $(
            impl FromBits<$n> for usize {
                fn from_bits(value: usize) -> Self {
                    value
                }
            }
        )*
    };
}

from_bits_impls!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64
);
