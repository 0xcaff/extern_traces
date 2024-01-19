use crate::instructions::formats::mubuf::Offset;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MTBUFOpCode;
use crate::instructions::operands::{ScalarGeneralPurposeRegisterGroup, VectorGPR};
use crate::{DisplayInstruction, DisplayableInstruction};
use bits::FromBits;
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct MTBufInstruction {
    #[bits(18, 16)]
    op: MTBUFOpCode,

    #[bits(11, 0)]
    offset: Offset,

    #[bits(12, 12)]
    offen: bool,

    #[bits(13, 13)]
    idxen: bool,

    #[bits(14, 14)]
    glc: bool,

    #[bits(15, 15)]
    addr64: bool,

    #[bits(22, 19)]
    dfmt: DataFormat,

    #[bits(25, 23)]
    nfmt: NumberFormat,

    #[bits(39, 32)]
    vaddr: u8,

    #[bits(47, 40)]
    vdata: VectorGPR,

    #[bits(52, 48)]
    srsrc: ScalarGeneralPurposeRegisterGroup,

    #[bits(54, 54)]
    slc: bool,

    #[bits(55, 55)]
    tfe: bool,

    #[bits(63, 56)]
    soffset: u8,
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

#[derive(Debug)]
struct NumberFormat(u8);

impl FromBits<3> for NumberFormat {
    fn from_bits(value: usize) -> Self {
        Self(value as _)
    }
}

#[derive(Debug)]
struct DataFormat(u8);

impl FromBits<4> for DataFormat {
    fn from_bits(value: usize) -> Self {
        DataFormat(value as _)
    }
}

impl<R: Reader> ParseInstruction<R> for MTBufInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MTBufInstruction::from_bits(token as usize))
    }
}

impl DisplayInstruction for MTBufInstruction {
    fn display(&self) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            // todo: figure out sizes
            args: vec!["SKIPPED".to_string()],
        }
    }
}
