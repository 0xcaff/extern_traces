use crate::instructions::generated::VOP1OpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use bits_macros::FromBits;

/// Vector Instruction One Input, One Output
///
/// Vector instruction taking one input and producing one output. Can be
/// followed by a 32-bit literal constant.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VOP1Instruction {
    #[bits(9, 16)]
    op: VOP1OpCode,

    #[bits(0, 8)]
    src0: SourceOperand,

    #[bits(17, 24)]
    vdst: VectorGPR,
}
