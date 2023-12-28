use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPPOpCode;
use crate::instructions::InstructionParseErrorKind;

/// Scalar Instruction One Input, One Special Operation
///
/// Scalar instruction taking one inline constant input and performing a
/// special operation (for example: jump).
pub struct SOPPInstruction {
    op: SOPPOpCode,

    simm16: u16,
}

impl<R: Reader> ParseInstruction<R> for SOPPInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(SOPPInstruction {
            op: SOPPOpCode::decode(bitrange(9, 7).of(token))?,
            simm16: bitrange(16, 16).of(token) as u16,
        })
    }
}
