use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VOP2OpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use bits::{bitrange, FromBits};

/// Vector Instruction Two Inputs, One Output
///
/// Vector instruction taking two inputs and producing one output. Can be
/// followed by a 32-bit literal constant.
#[derive(Debug)]
pub struct VOP2Instruction {
    op: VOP2OpCode,

    src0: SourceOperand,
    vsrc1: VectorGPR,
    vdst: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for VOP2Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(VOP2Instruction {
            op: VOP2OpCode::from_bits(bitrange(1, 6).of_32(token)),
            vdst: VectorGPR::from_bits(bitrange(7, 8).of_32(token)),
            vsrc1: VectorGPR::from_bits(bitrange(15, 8).of_32(token)),
            src0: SourceOperand::from_bits(bitrange(23, 9).of_32(token)),
        })
    }
}
