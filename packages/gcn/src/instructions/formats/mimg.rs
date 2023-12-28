use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::MIMGOpCode;
use crate::instructions::InstructionParseErrorKind;

pub struct MIMGInstruction {
    op: MIMGOpCode,
    // todo: remaining fields
}

impl<R: Reader> ParseInstruction<R> for MIMGInstruction {
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        let second_token = reader.read_u32()?;
        let token = ((first_token as u64) << 32) | second_token;

        Ok(MIMGInstruction {
            op: MIMGOpCode::decode(bitrange(7, 7).of(token))?,
        })
    }
}
