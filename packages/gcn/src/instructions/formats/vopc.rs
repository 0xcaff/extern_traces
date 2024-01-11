use crate::instructions::generated::VOPCOpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VOPCInstruction {
    #[bits(17, 24)]
    op: VOPCOpCode,

    #[bits(0, 8)]
    src0: SourceOperand,

    #[bits(9, 16)]
    vsrc1: VectorGPR,
}
