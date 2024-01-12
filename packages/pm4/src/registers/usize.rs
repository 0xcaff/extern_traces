use bits::FromBits;

#[derive(Debug)]
pub struct Usize(pub usize);

macro_rules! from_bits_impls {
    ($($n:expr),*) => {
        $(
            impl FromBits<$n> for Usize {
                fn from_bits(value: usize) -> Self {
                    Self(value as _)
                }
            }
        )*
    };
}

from_bits_impls!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32
);
