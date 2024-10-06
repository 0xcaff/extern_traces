use crate::instructions::display::DisplayInstruction;
use crate::instructions::formats::{combine, ParseInstruction};
use crate::instructions::operands::VectorGPR;
use crate::instructions::DisplayableInstruction;
use crate::SliceReader;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use bits::{bit, Bits, BitsContainerError, FromBits, TryFromBits, TryFromBitsContainer};
use bits_macros::TryFromBitsContainer;

/// Export
///
/// Export (output) pixel color, pixel depth, vertex position, or vertex
/// parameter data. Two words.
#[derive(Debug, TryFromBitsContainer)]
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

impl ParseInstruction for ExpInstruction {
    fn parse(token: u32, reader: &mut SliceReader) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(ExpInstruction::try_from_bits_container(token)?)
    }
}

fn vsrc<T: Bits>(idx: u8) -> impl Fn(T) -> Result<Option<VectorGPR>, BitsContainerError> {
    move |token: T| {
        let token = token.full();

        let compress = bit(10, token);
        let enabled = {
            if compress && idx >= 2 {
                false
            } else {
                bit(idx, token)
            }
        };

        if !enabled {
            return Ok(None);
        }

        Ok(Some(VectorGPR::from_bits(match idx {
            0 => token.slice(39, 32),
            1 => token.slice(47, 40),
            2 => token.slice(55, 48),
            3 => token.slice(63, 56),
            _ => return Err(BitsContainerError::new(0)),
        })))
    }
}

impl DisplayInstruction for ExpInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
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
    /// **M**ultiple **R**ender **T**arget (MRT) render target index
    RenderTarget(u8),
    Z,
    Null,
    Position(u8),
    Parameter(u8),
}

impl TryFromBits<6> for ExportTarget {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        let value = value.full();

        let result = match value {
            0..=7 => ExportTarget::RenderTarget(value as _),
            8 => ExportTarget::Z,
            9 => ExportTarget::Null,
            12..=15 => ExportTarget::Position((value - 12) as _),
            32..=63 => ExportTarget::Parameter((value - 32) as _),
            _ => return None,
        };

        Some(result)
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
