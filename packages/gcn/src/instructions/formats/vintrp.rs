use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VINTRPOpCode;
use crate::instructions::InstructionParseErrorKind;

pub struct VINTRPInstruction {
    op: VINTRPOpCode,
    // todo: implement
}

impl<R: Reader> ParseInstruction<R> for VINTRPInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(VINTRPInstruction {
            op: VINTRPOpCode::decode(bitrange(14, 2).of(token) as u8)?,
        })
    }
}
