use crate::format_pattern;
use crate::instructions::format_info::OpFormatPattern;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::InstructionParseErrorKind;

pub const PATTERN: OpFormatPattern = format_pattern!("111110");

// todo: implement
pub struct ExpInstruction {}

impl<R: Reader> ParseInstruction<R> for ExpInstruction {
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        let second_token = reader.read_u32()?;

        let token = ((first_token as u64) << 32) | second_token;

        Ok(ExpInstruction {})
    }
}
