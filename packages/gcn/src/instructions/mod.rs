mod bitrange;
mod formats;
mod generated;
mod operands;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io;
use std::io::{ErrorKind, Read};

use crate::instructions::formats::{FormattedInstruction, ParseInstruction};

pub struct Decoder<R> {
    reader: R,
}

#[derive(Debug)]
pub struct Instruction {
    inner: FormattedInstruction,
}

impl<R> Decoder<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Decoder<R> {
        Decoder { reader }
    }

    pub fn decode(&mut self) -> Result<Instruction, InstructionParseErrorKind> {
        let token = self.reader.read_u32::<LittleEndian>().wrapping_eof()?;

        let inner = FormattedInstruction::parse(token, &mut self.reader)?;

        Ok(Instruction { inner })
    }
}

pub enum InstructionParseErrorKind {
    Eof,
    Error(anyhow::Error),
}

trait ResultExt<T> {
    fn wrapping_eof(self) -> Result<T, InstructionParseErrorKind>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn wrapping_eof(self) -> Result<T, InstructionParseErrorKind> {
        self.map_err(|err| match err.kind() {
            ErrorKind::UnexpectedEof => InstructionParseErrorKind::Eof,
            _ => InstructionParseErrorKind::Error(err.into()),
        })
    }
}

impl<T> From<T> for InstructionParseErrorKind
where
    T: Into<anyhow::Error>,
{
    fn from(value: T) -> Self {
        InstructionParseErrorKind::Error(value.into())
    }
}
