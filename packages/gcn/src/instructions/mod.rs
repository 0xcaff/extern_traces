mod formats;
mod generated;
mod operands;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

use crate::instructions::formats::{FormattedInstruction, ParseInstruction};
use crate::reader::{ReadError, ResultExt};

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

    pub fn decode(&mut self) -> Result<Instruction, ReadError> {
        let token = self.reader.read_u32::<LittleEndian>().wrapping_eof()?;

        let inner = FormattedInstruction::parse(token, &mut self.reader)?;

        Ok(Instruction { inner })
    }
}
