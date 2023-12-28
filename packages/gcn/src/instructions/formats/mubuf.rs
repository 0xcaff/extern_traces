use crate::format_pattern;
use crate::instructions::format_info::{bitrange, OpFormatPattern};
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::MUBUFOpCode;
use crate::instructions::InstructionParseErrorKind;

pub const PATTERN: OpFormatPattern = format_pattern!("111000");

pub struct MUBUFInstruction {
    op: MUBUFOpCode,
}

impl<R: Reader> ParseInstruction<R> for MUBUFInstruction {
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        let second_token = reader.read_u32()?;

        let token = ((first_token as u64) << 32) | second_token;

        Ok(MUBUFInstruction {
            op: MUBUFOpCode::decode(bitrange(7, 7).of(token))?,
        })
    }
}
