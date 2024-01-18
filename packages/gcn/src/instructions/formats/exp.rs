use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::operands::VectorGPR;
use bits::{bit, bitrange, FromBits};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct ExpInstruction {
    #[bits(9, 4)]
    tgt: ExportTarget,

    #[bits(10, 10)]
    compr: bool,

    #[bits(11, 11)]
    done: bool,

    #[bits(12, 12)]
    vm: bool,

    #[bits(vsrc(0))]
    vsrc0: Option<VectorGPR>,

    #[bits(vsrc(1))]
    vsrc1: Option<VectorGPR>,

    #[bits(vsrc(2))]
    vsrc2: Option<VectorGPR>,

    #[bits(vsrc(3))]
    vsrc3: Option<VectorGPR>,
}

impl<R: Reader> ParseInstruction<R> for ExpInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(ExpInstruction::from_bits(token as usize))
    }
}

fn vsrc(idx: u8) -> impl Fn(usize) -> Option<VectorGPR> {
    move |token: usize| {
        let enabled = bit(idx, token);

        if !enabled {
            return None;
        }

        Some(VectorGPR::from_bits(
            (match idx {
                0 => bitrange(39, 32),
                1 => bitrange(47, 40),
                2 => bitrange(55, 48),
                3 => bitrange(63, 56),
                _ => unreachable!("invalid index"),
            })
            .of(token),
        ))
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
