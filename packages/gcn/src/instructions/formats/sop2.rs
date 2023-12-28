use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOP2OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use crate::instructions::InstructionParseErrorKind;

/// Scalar Format Two Inputs, One Output
///
/// This is a scalar instruction with two inputs and one output. Can be
/// followed by a 32-bit literal
pub struct SOP2Instruction {
    op: SOP2OpCode,

    ssrc0: ScalarSourceOperand,
    ssrc1: ScalarSourceOperand,
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOP2Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(SOP2Instruction {
            op: SOP2OpCode::decode(bitrange(2, 7).of(token))?,
            sdst: ScalarDestinationOperand::decode(bitrange(9, 7).of(token)),
            ssrc0: ScalarSourceOperand::decode(bitrange(24, 8).of(token)),
            ssrc1: ScalarSourceOperand::decode(bitrange(16, 8).of(token)),
        })
    }
}
