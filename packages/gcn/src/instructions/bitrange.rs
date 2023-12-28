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
