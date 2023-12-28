use crate::instructions::bitrange::bitrange;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::DSOpCode;
use crate::instructions::InstructionParseErrorKind;

#[derive(Debug)]
pub struct DSInstruction {
    op: DSOpCode,
    // todo: implement
    // offset0: u8,
    // offset1: u8,
    // gds: bool,

    // addr: VectorGPR,
    // data0: VectorGPR,
    // data1: VectorGPR,
    // vdst: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for DSInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, InstructionParseErrorKind> {
        let token = combine(token, reader)?;

        Ok(DSInstruction {
            op: DSOpCode::decode(bitrange(6, 8).of_64(token))?,
        })
    }
}
