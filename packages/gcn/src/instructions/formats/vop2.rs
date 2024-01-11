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
    #[bits(25, 30)]
    op: VOP2OpCode,

    #[bits(0, 8)]
    src0: SourceOperand,

    #[bits(9, 16)]
    vsrc1: VectorGPR,

    #[bits(17, 24)]
    vdst: VectorGPR,
}
