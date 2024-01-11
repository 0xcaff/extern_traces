use crate::instructions::generated::VOP2OpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use bits_macros::FromBits;

/// Vector Instruction Two Inputs, One Output
///
/// Vector instruction taking two inputs and producing one output. Can be
/// followed by a 32-bit literal constant.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VOP2Instruction {
    #[bits(30, 25)]
    op: VOP2OpCode,

    #[bits(8, 0)]
    src0: SourceOperand,

    #[bits(16, 9)]
    vsrc1: VectorGPR,

    #[bits(24, 17)]
    vdst: VectorGPR,
}
