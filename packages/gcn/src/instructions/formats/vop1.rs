use crate::bitrange::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VOP1OpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::reader::ReadError;

/// Vector Instruction One Input, One Output
///
/// Vector instruction taking one input and producing one output. Can be
/// followed by a 32-bit literal constant.
#[derive(Debug)]
pub struct VOP1Instruction {
    op: VOP1OpCode,

    src0: SourceOperand,
    vdst: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for VOP1Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, ReadError> {
        Ok(VOP1Instruction {
            op: VOP1OpCode::decode(bitrange(15, 8).of_32(token))?,
            vdst: VectorGPR::decode(bitrange(7, 8).of_32(token) as u8),
            src0: SourceOperand::decode(bitrange(23, 9).of_32(token) as u16),
        })
    }
}
