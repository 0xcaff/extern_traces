use crate::instructions::generated::SMEMOpCode;
use crate::instructions::operands::ScalarDestinationOperand;
use bits_macros::FromBits;

/// Scalar Instruction Memory Access
///
/// Scalar instruction performing a memory operation on scalar L1 memory. Two
/// Dwords.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SMEMInstruction {
    #[bits(26, 22)]
    op: SMEMOpCode,

    #[bits(7, 0)]
    offset: u8,

    #[bits(8, 8)]
    imm: bool,

    // todo:
    // #[bits(14, 9)]
    // sbase: u8,
    #[bits(21, 15)]
    sdst: ScalarDestinationOperand,
}
