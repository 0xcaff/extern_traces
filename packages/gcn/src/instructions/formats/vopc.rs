use crate::instructions::bitrange::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VOPCOpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::instructions::InstructionParseErrorKind;

#[derive(Debug)]
pub struct VOPCInstruction {
    op: VOPCOpCode,

    src0: SourceOperand,
    vsrc1: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for VOPCInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, InstructionParseErrorKind> {
        Ok(VOPCInstruction {
            op: VOPCOpCode::decode(bitrange(7, 8).of_32(token))?,
            vsrc1: VectorGPR::decode(bitrange(15, 8).of_32(token) as u8),
            src0: SourceOperand::decode(bitrange(23, 9).of_32(token) as u16),
        })
    }
}
