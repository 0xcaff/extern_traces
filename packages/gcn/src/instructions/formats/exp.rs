use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::operands::VectorGPR;
use bits::FromBits;
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct ExpInstruction {
    #[bits(3, 0)]
    en: En,

    #[bits(9, 4)]
    tgt: Tgt,

    #[bits(10, 10)]
    compr: bool,

    #[bits(11, 11)]
    done: bool,

    #[bits(12, 12)]
    vm: bool,

    #[bits(39, 32)]
    vsrc0: VectorGPR,

    #[bits(47, 40)]
    vsrc1: VectorGPR,

    #[bits(55, 48)]
    vsrc2: VectorGPR,

    #[bits(63, 56)]
    vsrc3: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for ExpInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(ExpInstruction::from_bits(token as usize))
    }
}

#[derive(Debug)]
struct En(u8);

impl FromBits<4> for En {
    fn from_bits(value: usize) -> Self {
        En(value as _)
    }
}

#[derive(Debug)]
struct Tgt(u8);

impl FromBits<6> for Tgt {
    fn from_bits(value: usize) -> Self {
        Tgt(value as _)
    }
}
