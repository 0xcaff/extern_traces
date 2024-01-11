use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOP1OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use bits::FromBits;
use bits_macros::FromBits;

/// Scalar Instruction One Input, One Output
///
/// This is a scalar instruction with one input and one output. Can be followed
/// by a 32-bit literal constant.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOP1Instruction {
    #[bits(8, 15)]
    op: SOP1OpCode,

    #[bits(0, 7)]
    ssrc0: ScalarSourceOperand,

    #[bits(16, 22)]
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOP1Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(Self::from_bits(token as usize))
    }
}
