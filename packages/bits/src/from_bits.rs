use crate::Bits;
use alloc::vec;
use alloc::vec::Vec;
use snafu::Snafu;

pub trait TryFromBits<const BITS: usize>: Sized {
    fn try_from_bits(value: impl Bits) -> Option<Self>;
}

pub trait FromBits<const BITS: usize>: Sized {
    fn from_bits(value: impl Bits) -> Self;
}

impl<const BITS: usize, T: FromBits<BITS>> TryFromBits<BITS> for T {
    fn try_from_bits(bits: impl Bits) -> Option<Self> {
        Some(Self::from_bits(bits))
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("failed to parse {:?}, got value {}", path, value))]
pub struct BitsContainerError {
    pub value: u64,
    pub path: Vec<&'static str>,
}

impl BitsContainerError {
    pub fn new(value: u64) -> BitsContainerError {
        BitsContainerError {
            path: vec![],
            value,
        }
    }

    pub fn extend(self, segment: &'static str) -> BitsContainerError {
        let mut path = self.path;
        path.push(segment);

        Self { path, value: self.value }
    }
}

pub trait TryFromBitsContainer<const BITS: usize>: Sized {
    fn try_from_bits_container(value: impl Bits) -> Result<Self, BitsContainerError>;
}

impl<const BITS: usize, T: TryFromBits<BITS>> TryFromBitsContainer<BITS> for T {
    fn try_from_bits_container(bits: impl Bits) -> Result<Self, BitsContainerError> {
        Ok(Self::try_from_bits(bits).ok_or_else(|| BitsContainerError::new(bits.full()))?)
    }
}

impl FromBits<16> for u16 {
    fn from_bits(value: impl Bits) -> Self {
        value.full() as _
    }
}

impl FromBits<16> for i16 {
    fn from_bits(value: impl Bits) -> Self {
        value.full() as _
    }
}

impl FromBits<8> for u8 {
    fn from_bits(value: impl Bits) -> Self {
        value.full() as _
    }
}

impl FromBits<1> for bool {
    fn from_bits(value: impl Bits) -> Self {
        value.full() != 0
    }
}

impl FromBits<32> for u32 {
    fn from_bits(value: impl Bits) -> Self {
        value.full() as _
    }
}

macro_rules! from_bits_impls {
    ($($n:expr),*) => {
        $(
            impl FromBits<$n> for u64 {
                fn from_bits(value: impl Bits) -> Self {
                    value.full()
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
