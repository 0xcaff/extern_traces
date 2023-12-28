use crate::format_pattern;
use crate::instructions::format_info::{bitrange, OpFormatPattern};
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VOP2OpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::instructions::InstructionParseErrorKind;

pub const FORMAT: OpFormatPattern = format_pattern!("0");

/// Vector Instruction Two Inputs, One Output
///
/// Vector instruction taking two inputs and producing one output. Can be
/// followed by a 32-bit literal constant.
pub struct VOP2Instruction {
    op: VOP2OpCode,

    src0: SourceOperand,
    vsrc1: VectorGPR,
    vdst: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for VOP2Instruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(VOP2Instruction {
            op: VOP2OpCode::decode(bitrange(1, 6).of(token) as u8)?,
            vdst: VectorGPR::decode(bitrange(7, 8).of(token) as u8),
            vsrc1: VectorGPR::decode(bitrange(15, 8).of(token) as u8),
            src0: SourceOperand::decode(bitrange(23, 9).of(token) as u16),
        })
    }
}
