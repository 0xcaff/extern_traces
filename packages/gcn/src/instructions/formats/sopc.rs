use crate::instructions::generated::SOPCOpCode;
use crate::instructions::operands::ScalarSourceOperand;
use bits_macros::FromBits;

/// Scalar Instruction Two Inputs, One Comparison
///
/// Scalar instruction taking two inputs and producing a comparison result. Can
/// be followed by a 32-bit literal constant
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPCInstruction {
    #[bits(22, 16)]
    op: SOPCOpCode,

    #[bits(7, 0)]
    ssrc0: ScalarSourceOperand,

    #[bits(15, 8)]
    ssrc1: ScalarSourceOperand,
}
