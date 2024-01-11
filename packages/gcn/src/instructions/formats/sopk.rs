use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPKOpCode;
use crate::instructions::operands::ScalarDestinationOperand;
use bits::{bitrange, FromBits};

/// Scalar Instruction One Inline Constant Input, One Output
///
/// This is a scalar instruction with one inline constant input and one output.
#[derive(Debug)]
pub struct SOPKInstruction {
    op: SOPKOpCode,

    simm16: u16,
    sdst: ScalarDestinationOperand,
}

impl<R: Reader> ParseInstruction<R> for SOPKInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(SOPKInstruction {
            op: SOPKOpCode::decode(bitrange(4, 5).of_32(token))?,
            simm16: bitrange(16, 16).of_32(token) as u16,
            sdst: ScalarDestinationOperand::from_bits(bitrange(9, 7).of_32(token)),
        })
    }
}
