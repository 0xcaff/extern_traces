use crate::instructions::display::DisplayInstruction;
use crate::instructions::formats::mubuf::Offset;
use crate::instructions::formats::{combine, ParseInstruction};
use crate::instructions::operands::{ScalarGeneralPurposeRegisterGroup, VectorGPR};
use crate::instructions::ops::MTBUFOpCode;
use crate::instructions::DisplayableInstruction;
use crate::SliceReader;
use alloc::string::ToString;
use alloc::vec;
use bits::{Bits, FromBits, TryFromBitsContainer};
use bits_macros::TryFromBitsContainer;

/// Typed Memory Buffer Operation
///
/// Typed memory buffer operation. Two words
#[derive(Debug, TryFromBitsContainer)]
#[bits(64)]
pub struct MTBufInstruction {
    #[bits(18, 16)]
    pub op: MTBUFOpCode,

    #[bits(11, 0)]
    pub offset: Offset,

    #[bits(12, 12)]
    pub offen: bool,

    #[bits(13, 13)]
    pub idxen: bool,

    #[bits(14, 14)]
    pub glc: bool,

    #[bits(15, 15)]
    pub addr64: bool,

    #[bits(22, 19)]
    pub dfmt: DataFormat,

    #[bits(25, 23)]
    pub nfmt: NumberFormat,

    #[bits(39, 32)]
    pub vaddr: u8,

    #[bits(47, 40)]
    pub vdata: VectorGPR,

    #[bits(52, 48)]
    pub srsrc: ScalarGeneralPurposeRegisterGroup,

    #[bits(54, 54)]
    pub slc: bool,

    #[bits(55, 55)]
    pub tfe: bool,

    #[bits(63, 56)]
    pub soffset: u8,
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
pub struct NumberFormat(pub u8);

impl FromBits<3> for NumberFormat {
    fn from_bits(value: impl Bits) -> Self {
        Self(value.full() as _)
    }
}

#[derive(Debug)]
pub struct DataFormat(pub u8);

impl FromBits<4> for DataFormat {
    fn from_bits(value: impl Bits) -> Self {
        DataFormat(value.full() as _)
    }
}

impl ParseInstruction for MTBufInstruction {
    fn parse(token: u32, reader: &mut SliceReader) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MTBufInstruction::try_from_bits_container(token)?)
    }
}

impl DisplayInstruction for MTBufInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            // todo: figure out sizes
            args: vec!["SKIPPED".to_string()],
        }
    }
}
