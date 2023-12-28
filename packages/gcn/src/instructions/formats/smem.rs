use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SMEMOpCode;
use crate::instructions::InstructionParseErrorKind;

/// Scalar Instruction Memory Access
///
/// Scalar instruction performing a memory operation on scalar L1 memory. Two
/// Dwords.
pub struct SMEMInstruction {
    op: SMEMOpCode,
    // todo: implement
}

impl<R: Reader> ParseInstruction<R> for SMEMInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(SMEMInstruction {
            op: SMEMOpCode::decode(bitrange(5, 5).of(token))?,
        })
    }
}
