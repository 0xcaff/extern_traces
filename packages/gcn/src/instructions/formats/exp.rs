use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::operands::VectorGPR;
use crate::{DisplayInstruction, DisplayableInstruction};
use bits::{bit, bitrange, FromBits};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct ExpInstruction {
    #[bits(9, 4)]
    pub target: ExportTarget,

    #[bits(10, 10)]
    pub compress: bool,

    #[bits(11, 11)]
    pub done: bool,

    #[bits(12, 12)]
    pub valid_mask: bool,

    #[bits(vsrc(0))]
    pub vsrc0: Option<VectorGPR>,

    #[bits(vsrc(1))]
    pub vsrc1: Option<VectorGPR>,

    #[bits(vsrc(2))]
    pub vsrc2: Option<VectorGPR>,

    #[bits(vsrc(3))]
    pub vsrc3: Option<VectorGPR>,
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

impl DisplayInstruction for ExpInstruction {
    fn display(&self) -> DisplayableInstruction {
        DisplayableInstruction {
            op: "exp".to_string(),
            args: {
                let mut values = vec![
                    self.target.display(),
                    self.vsrc0
                        .as_ref()
                        .map(|it| it.display(&None))
                        .unwrap_or("off".to_string()),
                    self.vsrc1
                        .as_ref()
                        .map(|it| it.display(&None))
                        .unwrap_or("off".to_string()),
                    self.vsrc2
                        .as_ref()
                        .map(|it| it.display(&None))
                        .unwrap_or("off".to_string()),
                    self.vsrc3
                        .as_ref()
                        .map(|it| it.display(&None))
                        .unwrap_or("off".to_string()),
                ];

                if self.compress {
                    values.push("compress".to_string());
                }

                if self.done {
                    values.push("done".to_string());
                }

                if self.valid_mask {
                    values.push("vm".to_string());
                }

                values
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExportTarget {
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

impl ExportTarget {
    fn display(&self) -> String {
        match self {
            ExportTarget::RenderTarget(idx) => format!("mrt_color{}", *idx),
            ExportTarget::Position(idx) => format!("pos{}", *idx),
            ExportTarget::Parameter(idx) => format!("param{}", *idx),
            ExportTarget::Z => "z".to_string(),
            ExportTarget::Null => "NULL".to_string(),
        }
    }
}
