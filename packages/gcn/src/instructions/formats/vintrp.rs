use crate::instructions::generated::VINTRPOpCode;
use crate::instructions::operands::VectorGPR;
use bits::FromBits;
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VINTRPInstruction {
    #[bits(17, 16)]
    op: VINTRPOpCode,

    #[bits(7, 0)]
    vsrc: VectorGPR,

    #[bits(9, 8)]
    attrchan: AttrChan,

    #[bits(15, 10)]
    attr: Attr,

    #[bits(25, 18)]
    vdst: VectorGPR,
}

#[derive(Debug)]
struct AttrChan(u8);

impl FromBits<2> for AttrChan {
    fn from_bits(value: usize) -> Self {
        AttrChan(value as _)
    }
}

#[derive(Debug)]
struct Attr(u8);

impl FromBits<6> for Attr {
    fn from_bits(value: usize) -> Self {
        Self(value as _)
    }
}
