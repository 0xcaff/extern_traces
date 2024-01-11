use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VOPCOpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use bits::{bitrange, FromBits};

#[derive(Debug)]
pub struct VOPCInstruction {
    op: VOPCOpCode,

    src0: SourceOperand,
    vsrc1: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for VOPCInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(VOPCInstruction {
            op: VOPCOpCode::decode(bitrange(7, 8).of_32(token))?,
            vsrc1: VectorGPR::from_bits(bitrange(15, 8).of_32(token)),
            src0: SourceOperand::from_bits(bitrange(23, 9).of_32(token)),
        })
    }
}
