use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MUBUFOpCode;
use crate::instructions::operands::{ScalarGeneralPurposeRegisterGroup, VectorGPR};
use crate::{DisplayInstruction, DisplayableInstruction};
use bits::{Bits, FromBits};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct MUBUFInstruction {
    #[bits(24, 18)]
    pub op: MUBUFOpCode,

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

    #[bits(16, 16)]
    pub lds: bool,

    #[bits(39, 32)]
    pub vaddr: VectorGPR,

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

#[derive(Debug)]
pub struct Offset(pub u16);

impl FromBits<12> for Offset {
    fn from_bits(value: impl Bits) -> Self {
        Offset(value.full() as u16)
    }
}

impl<R: Reader> ParseInstruction<R> for MUBUFInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MUBUFInstruction::from_bits(token))
    }
}

impl DisplayInstruction for MUBUFInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            // todo: figure out sizes
            args: vec!["SKIPPED".to_string()],
        }
    }
}
