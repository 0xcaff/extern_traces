use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOP2OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use bits::{bitrange, FromBits};

/// Scalar Format Two Inputs, One Output
///
/// This is a scalar instruction with two inputs and one output. Can be
/// followed by a 32-bit literal
#[derive(Debug)]
pub struct SOP2Instruction {
    op: SOP2OpCode,

    ssrc0: ScalarSourceOperand,
    ssrc1: ScalarSourceOperand,
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOP2Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(SOP2Instruction {
            op: SOP2OpCode::decode(bitrange(2, 7).of_32(token))?,
            sdst: ScalarDestinationOperand::from_bits(bitrange(9, 7).of_32(token)),
            ssrc0: ScalarSourceOperand::from_bits(bitrange(24, 8).of_32(token)),
            ssrc1: ScalarSourceOperand::from_bits(bitrange(16, 8).of_32(token)),
        })
    }
}
