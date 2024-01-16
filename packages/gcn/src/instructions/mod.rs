mod formats;
mod generated;
mod operands;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{BufRead, BufReader, Read};

use crate::instructions::formats::{FormattedInstruction, ParseInstruction};

pub struct Decoder<R> {
    reader: BufReader<R>,
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
        Decoder {
            reader: BufReader::new(reader),
        }
    }

    pub fn decode(&mut self) -> Result<Option<Instruction>, anyhow::Error> {
        // todo: rework this, doesn't handle branching or termination correctly
        if !self.reader.has_data_left()? {
            return Ok(None);
        }

        let token = self.reader.read_u32::<LittleEndian>()?;

        let inner = FormattedInstruction::parse(token, &mut self.reader)?;

        Ok(Some(Instruction { inner }))
    }
}
