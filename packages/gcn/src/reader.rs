use byteorder::{LittleEndian, ReadBytesExt};
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

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, ReadError>;
}

impl<R: Read> Reader for R {
    fn read_u32(&mut self) -> Result<u32, ReadError> {
        Ok(ReadBytesExt::read_u32::<LittleEndian>(self).wrapping_eof()?)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, ReadError> {
        let mut buffer = vec![0; len];

        self.read_exact(&mut buffer).wrapping_eof()?;

        Ok(buffer)
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
