use crate::instructions::display::DisplayInstruction;
use crate::instructions::formats::{combine, ParseInstruction};
use crate::instructions::instruction_info::OperandInfo;
use crate::instructions::operands::ScalarDestinationOperand;
use crate::instructions::operands::{ScalarGeneralPurposeRegisterGroup, VectorGPR};
use crate::instructions::ops::MIMGOpCode;
use crate::instructions::DisplayableInstruction;
use crate::SliceReader;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use bits::{bit, Bits, FromBits, TryFromBitsContainer};
use bits_macros::TryFromBitsContainer;
use core::fmt;

/// Image Memory Buffer Operations
///
/// Image memory buffer operations. Two words.
#[derive(Debug, TryFromBitsContainer)]
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

pub struct DMask(u8);

impl FromBits<4> for DMask {
    fn from_bits(value: impl Bits) -> Self {
        DMask(value.full() as _)
    }
}

impl DMask {
    pub fn indices(&self) -> impl Iterator<Item = u8> + '_ {
        (0..4).into_iter().flat_map(|idx| {
            let enabled = bit(idx, self.0 as _);
            if !enabled {
                return None;
            }

            return Some(idx);
        })
    }
}

impl fmt::Debug for DMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "dmask({})",
            self.indices()
                .map(|it| match it {
                    0 => "r",
                    1 => "g",
                    2 => "b",
                    3 => "a",
                    _ => unreachable!(),
                })
                .collect::<String>(),
        ))
    }
}

impl ParseInstruction for MIMGInstruction {
    fn parse(token: u32, reader: &mut SliceReader) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MIMGInstruction::try_from_bits_container(token)?)
    }
}

impl DisplayInstruction for MIMGInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        let indices = self.dmask.indices().collect::<Vec<_>>();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: {
                let mut args = vec![
                    self.vdata
                        .display(&Some(OperandInfo::Size(indices.len() as u8))),
                    self.vaddr.display(&Some(OperandInfo::Size(2))),
                    ScalarDestinationOperand::ScalarGPR(self.srsrc.lowest_register_idx())
                        .display(&Some(OperandInfo::Size(if self.r128 { 4 } else { 8 }))),
                    self.ssamp.display(),
                    format!("{:?}", self.dmask),
                ];

                if self.unorm {
                    args.push("unorm".into());
                }

                if self.glc {
                    args.push("glc".into());
                }

                if self.da {
                    args.push("da".into());
                }

                if self.r128 {
                    args.push("r128".into());
                }

                if self.tfe {
                    args.push("tfe".into());
                }

                if self.lwe {
                    args.push("lwe".into());
                }

                if self.slc {
                    args.push("slc".into());
                }

                args
            },
        }
    }
}
