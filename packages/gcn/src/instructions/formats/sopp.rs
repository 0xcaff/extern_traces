use crate::bitrange::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPPOpCode;

/// Scalar Instruction One Input, One Special Operation
///
/// Scalar instruction taking one inline constant input and performing a
/// special operation (for example: jump).
#[derive(Debug)]
pub struct SOPPInstruction {
    op: SOPPOpCode,

    simm16: u16,
}

impl<R: Reader> ParseInstruction<R> for SOPPInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(SOPPInstruction {
            op: SOPPOpCode::decode(bitrange(9, 7).of_32(token))?,
            simm16: bitrange(16, 16).of_32(token) as u16,
        })
    }
}
