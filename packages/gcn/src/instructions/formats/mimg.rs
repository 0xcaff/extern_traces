use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MIMGOpCode;
use crate::instructions::operands::VectorGPR;
use bits::FromBits;
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct MIMGInstruction {
    #[bits(24, 18)]
    op: MIMGOpCode,

    #[bits(11, 8)]
    dmask: DMask,

    #[bits(12, 12)]
    unorm: bool,

    #[bits(13, 13)]
    glc: bool,

    #[bits(14, 14)]
    da: bool,

    #[bits(15, 15)]
    r128: bool,

    #[bits(16, 16)]
    tfe: bool,

    #[bits(17, 17)]
    lwe: bool,

    #[bits(25, 25)]
    slc: bool,

    #[bits(39, 32)]
    vaddr: VectorGPR,

    #[bits(47, 40)]
    vdata: VectorGPR,
    // todo:
    // #[bits(52, 48)]
    // srsrc: u8,

    // todo:
    // #[bits(57, 53)]
    // ssam: u8,
}

#[derive(Debug)]
struct DMask(u8);

impl FromBits<4> for DMask {
    fn from_bits(value: usize) -> Self {
        DMask(value as _)
    }
}

impl<R: Reader> ParseInstruction<R> for MIMGInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MIMGInstruction::from_bits(token as usize))
    }
}
