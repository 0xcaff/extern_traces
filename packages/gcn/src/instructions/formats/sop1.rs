use crate::instructions::bitrange::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOP1OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use crate::instructions::InstructionParseErrorKind;

/// Scalar Instruction One Input, One Output
///
/// This is a scalar instruction with one input and one output. Can be followed
/// by a 32-bit literal constant.
#[derive(Debug)]
pub struct SOP1Instruction {
    op: SOP1OpCode,

    ssrc0: ScalarSourceOperand,
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOP1Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(SOP1Instruction {
            op: SOP1OpCode::decode(bitrange(16, 8).of_32(token))?,
            sdst: ScalarDestinationOperand::decode(bitrange(9, 7).of_32(token) as u8),
            ssrc0: ScalarSourceOperand::decode(bitrange(24, 8).of_32(token ) as u8),
        })
    }
}
