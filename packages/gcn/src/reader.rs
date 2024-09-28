use snafu::Snafu;

#[derive(Debug, Snafu)]
pub struct EofError;

pub struct SliceReader<'a> {
    slice: &'a [u32],
    offset: usize,
}

impl SliceReader<'_> {
}

impl<'a> SliceReader<'a> {
    pub fn new(slice: &'a [u32]) -> SliceReader<'a> {
        SliceReader { slice, offset: 0 }
    }

    pub fn has_more(&self) -> bool {
        !self.slice.is_empty()
    }
    
    pub fn position(&self) -> usize {
        self.offset
    }
    
    pub fn read_u32(&mut self) -> Result<u32, EofError> {
        if self.slice.is_empty() {
            return Err(EofError);
        }

        let value = self.slice[0];
        self.slice = &self.slice[1..];
        self.offset += 1;

        Ok(value)
    }
}
