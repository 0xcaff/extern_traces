use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VOPCOpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::instructions::InstructionParseErrorKind;

pub struct VOPCInstruction {
    op: VOPCOpCode,

    src0: SourceOperand,
    vsrc1: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for VOPCInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(VOPCInstruction {
            op: VOPCOpCode::decode(bitrange(7, 8).of(token) as u8)?,
            vsrc1: VectorGPR::decode(bitrange(15, 8).of(token) as u8),
            src0: SourceOperand::decode(bitrange(23, 9).of(token) as u16),
        })
    }
}
