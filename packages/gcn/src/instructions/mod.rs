mod control_flow;
mod formats;
mod generated;
mod operands;

use std::io::{BufRead, BufReader, Read};

use crate::reader::Reader;

pub use control_flow::*;
pub use formats::*;
pub use generated::*;

pub struct Decoder<R> {
    reader: BufReader<R>,
}

#[derive(Debug)]
pub struct Instruction {
    pub inner: FormattedInstruction,
    program_counter: u64,
}

impl Instruction {
    pub fn parse(
        mut reader: impl Read,
        program_counter: u64,
    ) -> Result<Instruction, anyhow::Error> {
        let token = reader.read_u32()?;

        Ok(Self {
            inner: FormattedInstruction::parse(token, &mut reader)?,
            program_counter,
        })
    }
}
