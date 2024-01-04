use crate::bitrange::bitrange;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MTBUFOpCode;
use crate::reader::ReadError;

#[derive(Debug)]
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
    fn parse(token: u32, reader: R) -> Result<Self, ReadError> {
        let token = combine(token, reader)?;
        Ok(MTBufInstruction {
            op: MTBUFOpCode::decode(bitrange(13, 3).of_64(token))?,
        })
    }
}
