use crate::instructions::generated::SOPKOpCode;
use crate::instructions::operands::ScalarDestinationOperand;
use bits_macros::FromBits;

/// Scalar Instruction One Inline Constant Input, One Output
///
/// This is a scalar instruction with one inline constant input and one output.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPKInstruction {
    #[bits(27, 23)]
    op: SOPKOpCode,

    #[bits(15, 0)]
    simm16: u16,

    #[bits(22, 16)]
    sdst: ScalarDestinationOperand,
}
