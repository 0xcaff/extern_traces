use crate::instructions::formats::ParseInstruction;
use crate::instructions::generated::SOP2OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use crate::reader::Reader;
use bits::FromBits;
use bits_macros::FromBits;

/// Scalar Format Two Inputs, One Output
///
/// This is a scalar instruction with two inputs and one output. Can be
/// followed by a 32-bit literal
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOP2Instruction {
    #[bits(23, 29)]
    op: SOP2OpCode,

    #[bits(0, 7)]
    ssrc0: ScalarSourceOperand,

    #[bits(8, 15)]
    ssrc1: ScalarSourceOperand,

    #[bits(16, 22)]
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOP2Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(Self::from_bits(token as usize))
    }
}
