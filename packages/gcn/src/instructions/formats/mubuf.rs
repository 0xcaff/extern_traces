use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MUBUFOpCode;
use crate::instructions::operands::VectorGPR;
use bits::FromBits;
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct MUBUFInstruction {
    #[bits(24, 18)]
    op: MUBUFOpCode,

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

    #[bits(16, 16)]
    lds: bool,

    #[bits(39, 32)]
    vaddr: VectorGPR,

    #[bits(47, 40)]
    vdata: VectorGPR,

    // todo:
    // #[bits(52, 48)]
    // srsrc: ScalarSourceOperand
    #[bits(54, 54)]
    slc: bool,

    #[bits(55, 55)]
    tfe: bool,

    #[bits(63, 56)]
    soffset: u8,
}

#[derive(Debug)]
pub struct Offset(u16);

impl FromBits<12> for Offset {
    fn from_bits(value: usize) -> Self {
        Offset(value as u16)
    }
}

impl<R: Reader> ParseInstruction<R> for MUBUFInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MUBUFInstruction::from_bits(token as usize))
    }
}
