use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPPOpCode;
use bits::FromBits;
use bits_macros::FromBits;

/// Scalar Instruction One Input, One Special Operation
///
/// Scalar instruction taking one inline constant input and performing a
/// special operation (for example: jump).
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPPInstruction {
    #[bits(16, 22)]
    op: SOPPOpCode,

    #[bits(0, 15)]
    simm16: u16,
}

impl<R: Reader> ParseInstruction<R> for SOPPInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(Self::from_bits(token as usize))
    }
}
