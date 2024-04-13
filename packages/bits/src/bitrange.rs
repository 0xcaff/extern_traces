/// Holds information to extract a set of bits from a value.
///
/// The start value is relative to the least significant bit, so a start of 0
/// will start from the most significant bit. The least significant bit index
/// is the only stable anchor point to index against.
#[derive(PartialEq, Eq, Debug)]
pub struct BitRange {
    least_significant_idx: u8,
    mask: u64,
}

impl BitRange {
    pub fn of(&self, value: u64) -> u64 {
        (value >> self.least_significant_idx) & self.mask
    }

    pub fn of_32(&self, value: u32) -> usize {
        self.of(value as _) as _
    }
}

/// Constructs a bit range spanning from [`most_significant_idx`] to
/// [`least_significant_idx`], inclusive of both the upper and lower bounds.
/// Bits will maintain the ordering of their input.
pub const fn bitrange(most_significant_idx: u8, least_significant_idx: u8) -> BitRange {
    let len = most_significant_idx - least_significant_idx + 1;
    let mask = ((1u64) << len) - 1;

    BitRange {
        least_significant_idx,
        mask,
    }
}

pub fn bit(idx: u8, of: u64) -> bool {
    match bitrange(idx, idx).of(of) {
        0 => false,
        1 => true,
        value => unreachable!("unknown value {}", value),
    }
}

#[cfg(test)]
mod tests {
    use crate::bitrange;

    #[test]
    fn test() {
        let value = 0b11010000000110001000001110101011;

        assert_eq!(bitrange(31, 26).of(value), 0b110100);
        assert_eq!(bitrange(31, 26).of(value), 0b000000);
    }
}
