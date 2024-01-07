use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::io;
use std::io::{ErrorKind, Read};

pub trait ResultExt<T> {
    fn wrapping_eof(self) -> Result<T, ReadError>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn wrapping_eof(self) -> Result<T, ReadError> {
        self.map_err(|err| match err.kind() {
            ErrorKind::UnexpectedEof => ReadError::Eof,
            _ => ReadError::Error(err.into()),
        })
    }
}

pub trait Reader {
    fn read_u32(&mut self) -> Result<u32, ReadError>;

    fn read_dwords(&mut self, len: usize) -> Result<Vec<u32>, ReadError>;
}

impl<R: Read> Reader for R {
    fn read_u32(&mut self) -> Result<u32, ReadError> {
        Ok(ReadBytesExt::read_u32::<LittleEndian>(self).wrapping_eof()?)
    }

    fn read_dwords(&mut self, len: usize) -> Result<Vec<u32>, ReadError> {
        let mut buffer = vec![0; len * 4];

        self.read_exact(&mut buffer).wrapping_eof()?;

        Ok(buffer
            .chunks(4)
            .map(|it| LittleEndian::read_u32(it))
            .collect::<Vec<u32>>())
    }
}

pub enum ReadError {
    Eof,
    Error(anyhow::Error),
}

impl<T> From<T> for ReadError
where
    T: Into<anyhow::Error>,
{
    fn from(value: T) -> Self {
        ReadError::Error(value.into())
    }
}
