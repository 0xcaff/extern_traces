use crate::instructions::formats::{combine, ParseInstruction, Reader};

// todo: implement
#[derive(Debug)]
pub struct ExpInstruction {}

impl<R: Reader> ParseInstruction<R> for ExpInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(ExpInstruction {})
    }
}
