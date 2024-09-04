use byteorder::{LittleEndian, ReadBytesExt};
use std::io;
use std::io::Read;

pub trait Reader {
    fn read_u32(&mut self) -> Result<u32, io::Error>;
}

impl<R: Read> Reader for R {
    fn read_u32(&mut self) -> Result<u32, io::Error> {
        Ok(ReadBytesExt::read_u32::<LittleEndian>(self)?)
    }
}

pub struct SliceReader<'a> {
    pub slice: &'a [u32],
}

impl SliceReader<'_> {
    pub fn new(slice: &[u32]) -> SliceReader {
        SliceReader { slice }
    }
}

impl<'a> Reader for SliceReader<'a> {
    fn read_u32(&mut self) -> Result<u32, io::Error> {
        if self.slice.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let value = self.slice[0];
        self.slice = &self.slice[1..];

        Ok(value)
    }
}
