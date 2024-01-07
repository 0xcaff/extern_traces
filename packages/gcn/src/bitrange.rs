/// Holds information to extract a set of bits from a value.
///
/// The start value is relative to the most significant bit, so a start of 0
/// will start from the most significant bit.
#[derive(PartialEq, Eq, Debug)]
pub struct BitRange {
    pub start: u8,
    pub len: u8,
}

impl BitRange {
    pub fn of_32(&self, value: u32) -> usize {
        let offset = 32 - self.len - self.start;

        let mask = ((1 << self.len) - 1) << offset;

        ((mask & value) >> offset) as usize
    }

    pub fn of_64(&self, value: u64) -> usize {
        let offset = 64 - self.len - self.start;

        let mask = ((1 << self.len) - 1) << offset;

        ((mask & value) >> offset) as usize
    }
}

pub const fn bitrange(start_idx: u8, len: u8) -> BitRange {
    BitRange {
        start: start_idx,
        len,
    }
}

#[cfg(test)]
mod tests {
    use crate::bitrange::bitrange;

    #[test]
    fn test() {
        let value = 0b11010000000110001000001110101011u32;

        assert_eq!(bitrange(0, 6).of_32(value), 0b110100);
        assert_eq!(bitrange(0, 6).of_64(value as u64), 0b000000);
    }
}
