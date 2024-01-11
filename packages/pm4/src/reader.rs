use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::io;
use std::io::Read;

pub trait Reader {
    fn read_u32(&mut self) -> Result<u32, io::Error>;

    fn read_dwords(&mut self, len: usize) -> Result<Vec<u32>, io::Error>;
}

impl<R: Read> Reader for R {
    fn read_u32(&mut self) -> Result<u32, io::Error> {
        Ok(ReadBytesExt::read_u32::<LittleEndian>(self)?)
    }

    fn read_dwords(&mut self, len: usize) -> Result<Vec<u32>, io::Error> {
        let mut buffer = vec![0; len * 4];

        self.read_exact(&mut buffer)?;

        Ok(buffer
            .chunks(4)
            .map(|it| LittleEndian::read_u32(it))
            .collect::<Vec<u32>>())
    }
}
