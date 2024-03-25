mod control_flow;
mod display;
mod formats;
mod generated;
mod instruction_info;
mod operands;

use std::io::Read;

use crate::reader::Reader;

pub use control_flow::*;
pub use display::*;
pub use formats::*;
pub use generated::*;
pub use operands::*;

#[derive(Debug)]
pub struct Instruction {
    pub inner: FormattedInstruction,
    pub literal_constant: Option<u32>,
    pub program_counter: u64,
}

impl Instruction {
    pub fn parse(
        mut reader: impl Read,
        program_counter: u64,
    ) -> Result<Instruction, anyhow::Error> {
        let token = reader.read_u32()?;

        let inner = FormattedInstruction::parse(token, &mut reader)?;

        let literal_constant = if inner.has_literal_constant() {
            Some(reader.read_u32()?)
        } else {
            None
        };

        Ok(Self {
            inner,
            literal_constant,
            program_counter,
        })
    }
}
