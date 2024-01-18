use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::operands::VectorGPR;
use bits::FromBits;
use bits_macros::FromBits;
use std::fmt;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct ExpInstruction {
    #[bits(3, 0)]
    en: EnabledExports,

    #[bits(9, 4)]
    tgt: ExportTarget,

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

// todo: flatten this
#[derive(Debug, FromBits)]
#[bits(4)]
struct EnabledExports {
    #[bits(0, 0)]
    vsrc0: bool,

    #[bits(1, 1)]
    vsrc1: bool,

    #[bits(2, 2)]
    vsrc2: bool,

    #[bits(3, 3)]
    vsrc3: bool,
}

#[derive(Debug)]
enum ExportTarget {
    RenderTarget(u8),
    Z,
    Null,
    Position(u8),
    Parameter(u8),
}

impl FromBits<6> for ExportTarget {
    fn from_bits(value: usize) -> Self {
        match value {
            0..=7 => ExportTarget::RenderTarget(value as _),
            8 => ExportTarget::Z,
            9 => ExportTarget::Null,
            12..=15 => ExportTarget::Position((value - 12) as _),
            32..=63 => ExportTarget::Parameter((value - 32) as _),
            _ => panic!("unexpected value {}", value),
        }
    }
}

impl fmt::Display for ExportTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportTarget::RenderTarget(idx) => write!(f, "mrt_color{}", *idx),
            ExportTarget::Position(idx) => write!(f, "pos{}", *idx),
            ExportTarget::Parameter(idx) => write!(f, "param{}", *idx),
            ExportTarget::Z => write!(f, "z"),
            ExportTarget::Null => write!(f, "NULL"),
        }
    }
}
