use std::io;
use std::io::Read;

pub trait ReadExt {
    fn read_all(&mut self) -> io::Result<Vec<u8>>;
}

impl<R> ReadExt for R
where
    R: Read,
{
    fn read_all(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}
