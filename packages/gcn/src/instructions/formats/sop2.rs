use crate::instructions::generated::SOP2OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use bits_macros::FromBits;

/// Scalar Format Two Inputs, One Output
///
/// This is a scalar instruction with two inputs and one output. Can be
/// followed by a 32-bit literal
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOP2Instruction {
    #[bits(29, 23)]
    op: SOP2OpCode,

    #[bits(7, 0)]
    ssrc0: ScalarSourceOperand,

    #[bits(15, 8)]
    ssrc1: ScalarSourceOperand,

    #[bits(22, 16)]
    sdst: ScalarDestinationOperand,
}
