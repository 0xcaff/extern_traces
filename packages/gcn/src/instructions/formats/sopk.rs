use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPKOpCode;
use crate::instructions::operands::ScalarDestinationOperand;
use bits::FromBits;
use bits_macros::FromBits;

/// Scalar Instruction One Inline Constant Input, One Output
///
/// This is a scalar instruction with one inline constant input and one output.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPKInstruction {
    #[bits(23, 27)]
    op: SOPKOpCode,

    #[bits(0, 15)]
    simm16: u16,

    #[bits(16, 22)]
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOPKInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(Self::from_bits(token as usize))
    }
}
