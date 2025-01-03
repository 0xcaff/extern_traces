pub mod control_flow;
pub mod display;
pub mod formats;

mod instruction_info;
pub mod operands;
#[path = "generated/mod.rs"]
pub mod ops;

use crate::instructions::display::{DisplayInstruction, DisplayableInstruction};
use crate::instructions::formats::{FormattedInstruction, ParseInstruction};
use crate::SliceReader;

#[derive(Debug)]
pub struct Instruction {
    pub inner: FormattedInstruction,
    pub literal_constant: Option<u32>,
    pub program_counter: u64,
}

impl Instruction {
    pub fn display(&self) -> DisplayableInstruction {
        self.inner.display(self.literal_constant)
    }

    pub fn parse(
        reader: &mut SliceReader,
        program_counter: u64,
    ) -> Result<Instruction, anyhow::Error> {
        let token = reader.read_u32()?;
        let inner = FormattedInstruction::parse(token, reader)?;

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
