use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::InstructionParseErrorKind;

// todo: implement
pub struct ExpInstruction {}

impl<R: Reader> ParseInstruction<R> for ExpInstruction {
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        let second_token = reader.read_u32()?;

        let token = ((first_token as u64) << 32) | second_token;

        Ok(ExpInstruction {})
    }
}
