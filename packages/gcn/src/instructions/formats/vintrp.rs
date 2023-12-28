use crate::instructions::bitrange::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VINTRPOpCode;
use crate::instructions::InstructionParseErrorKind;

#[derive(Debug)]
pub struct VINTRPInstruction {
    op: VINTRPOpCode,
    // todo: implement
}

impl<R: Reader> ParseInstruction<R> for VINTRPInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(VINTRPInstruction {
            op: VINTRPOpCode::decode(bitrange(14, 2).of_32(token))?,
        })
    }
}
