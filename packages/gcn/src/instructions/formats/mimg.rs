use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MIMGOpCode;
use crate::instructions::operands::{ScalarGeneralPurposeRegisterGroup, VectorGPR};
use crate::{DisplayInstruction, DisplayableInstruction};
use bits::{Bits, FromBits};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct MIMGInstruction {
    #[bits(24, 18)]
    pub op: MIMGOpCode,

    #[bits(11, 8)]
    pub dmask: DMask,

    #[bits(12, 12)]
    pub unorm: bool,

    #[bits(13, 13)]
    pub glc: bool,

    #[bits(14, 14)]
    pub da: bool,

    #[bits(15, 15)]
    pub r128: bool,

    #[bits(16, 16)]
    pub tfe: bool,

    #[bits(17, 17)]
    pub lwe: bool,

    #[bits(25, 25)]
    pub slc: bool,

    #[bits(39, 32)]
    pub vaddr: VectorGPR,

    #[bits(47, 40)]
    pub vdata: VectorGPR,

    #[bits(52, 48)]
    pub srsrc: ScalarGeneralPurposeRegisterGroup,

    #[bits(57, 53)]
    pub ssamp: ScalarGeneralPurposeRegisterGroup,
}

#[derive(Debug)]
struct DMask(u8);

impl FromBits<4> for DMask {
    fn from_bits(value: impl Bits) -> Self {
        DMask(value.full() as _)
    }
}

impl<R: Reader> ParseInstruction<R> for MIMGInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MIMGInstruction::from_bits(token))
    }
}

impl DisplayInstruction for MIMGInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                // todo: figure out sizes
                "SKIPPED".to_string(),
            ],
        }
    }
}
