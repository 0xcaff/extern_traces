use crate::instructions::generated::SOP1OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use bits_macros::FromBits;

/// Scalar Instruction One Input, One Output
///
/// This is a scalar instruction with one input and one output. Can be followed
/// by a 32-bit literal constant.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOP1Instruction {
    #[bits(15, 8)]
    pub op: SOP1OpCode,

    #[bits(7, 0)]
    pub ssrc0: ScalarSourceOperand,

    #[bits(22, 16)]
    pub sdst: ScalarDestinationOperand,
}
