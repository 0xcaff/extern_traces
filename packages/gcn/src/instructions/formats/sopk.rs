use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPKOpCode;
use crate::instructions::operands::ScalarDestinationOperand;
use crate::instructions::InstructionParseErrorKind;

/// Scalar Instruction One Inline Constant Input, One Output
///
/// This is a scalar instruction with one inline constant input and one output.
pub struct SOPKInstruction {
    op: SOPKOpCode,

    simm16: u16,
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOPKInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(SOPKInstruction {
            op: SOPKOpCode::decode(bitrange(4, 5).of(token))?,
            simm16: bitrange(16, 16).of(token) as u16,
            sdst: ScalarDestinationOperand::decode(bitrange(9, 7).of(token)),
        })
    }
}
