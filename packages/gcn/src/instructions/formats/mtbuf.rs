use crate::instructions::format_info::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::MTBUFOpCode;
use crate::instructions::InstructionParseErrorKind;

pub struct MTBufInstruction {
    op: MTBUFOpCode,
    // todo: implement
    // offset: u16,
    // offen: bool,
    // idxen: bool,
    // glc: bool,

    // data_format: DataFormat,
    // number_format: NumberFormat,

    // vaddr: u8,
    // vdata: VectorGPR,
    // // todo:
    // srsrc: u8,
    // slc: bool,
    // tfe: bool,

    // soffset: u8,
}

// enum DataFormat {
//     // todo: implement
// }
//
// enum NumberFormat {
//     Unorm = 0,
//     Snorm = 1,
//     Uscaled = 2,
//     Sscaled = 3,
//     Uint = 4,
//     Sint = 5,
//     Reserved = 6,
//     Float = 7,
// }

impl<R: Reader> ParseInstruction<R> for MTBufInstruction {
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        let second_token = reader.read_u32()?;

        let token = ((first_token as u64) << 32) | second_token;

        Ok(MTBufInstruction {
            op: MTBUFOpCode::decode(bitrange(13, 3).of(token))?,
        })
    }
}
