use crate::format_pattern;
use crate::instructions::format_info::{bitrange, OpFormatPattern};
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::DSOpCode;
use crate::instructions::InstructionParseErrorKind;

pub const PATTERN: OpFormatPattern = format_pattern!("110110");

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
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        // todo: refactor this
        let second_token = reader.read_u32()?;
        let token = ((first_token as u64) << 32) | second_token;

        Ok(DSInstruction {
            op: DSOpCode::decode(bitrange(6, 8).of(token))?,
        })
    }
}
