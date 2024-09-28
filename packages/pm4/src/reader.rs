use alloc::vec::Vec;

pub struct Reader<'a> {
    inner: &'a [u8],
    cursor: usize,
}

impl <'a> Reader<'a> {
    pub fn new(inner: &'a [u8]) -> Reader<'a> {
        Reader {
            inner,
            cursor: 0,
        }
    }
    
    pub fn has_more(&self) -> bool {
         self.cursor < self.inner.len()
    }
    
    pub fn read_u32(&mut self) -> Option<u32> {
        let bytes: [u8; 4] = self.inner[self.cursor..(self.cursor + 4)].try_into().ok()?;
        self.cursor += 4;
        Some(u32::from_le_bytes(bytes))
    }

    pub fn read_dwords(&mut self, len: usize) -> Option<Vec<u32>> {
        let mut result = Vec::with_capacity(len);
        for _ in 0..len {
            result.push(self.read_u32()?);
        }
        Some(result)
    }
}
