use crate::instructions::generated::VOPCOpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VOPCInstruction {
    #[bits(24, 17)]
    op: VOPCOpCode,

    #[bits(8, 0)]
    src0: SourceOperand,

    #[bits(16, 9)]
    vsrc1: VectorGPR,
}
