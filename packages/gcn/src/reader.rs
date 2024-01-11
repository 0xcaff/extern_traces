use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
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
