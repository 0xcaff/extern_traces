use crate::instructions::display::DisplayInstruction;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::instruction_info::OperandInfo;
use crate::instructions::operands::{ScalarGeneralPurposeRegisterGroup, VectorGPR};
use crate::instructions::ops::MUBUFOpCode;
use crate::instructions::DisplayableInstruction;
use bits::{Bits, FromBits};
use bits_macros::FromBits;

/// Untyped Vector Memory Buffer Operation
///
/// Untyped memory buffer operation. First word with LDS, second word non-LDS.
#[derive(Debug, FromBits)]
#[bits(64)]
pub struct MUBUFInstruction {
    #[bits(24, 18)]
    pub op: MUBUFOpCode,

    #[bits(11, 0)]
    pub offset: Offset,

    #[bits(12, 12)]
    pub offen: bool,

    #[bits(13, 13)]
    pub idxen: bool,

    #[bits(14, 14)]
    pub glc: bool,

    #[bits(15, 15)]
    pub addr64: bool,

    #[bits(16, 16)]
    pub lds: bool,

    #[bits(39, 32)]
    pub vaddr: VectorGPR,

    #[bits(47, 40)]
    pub vdata: VectorGPR,

    #[bits(52, 48)]
    pub srsrc: ScalarGeneralPurposeRegisterGroup,

    #[bits(54, 54)]
    pub slc: bool,

    #[bits(55, 55)]
    pub tfe: bool,

    #[bits(63, 56)]
    pub soffset: u8,
}

#[derive(Debug)]
pub struct Offset(pub u16);

impl FromBits<12> for Offset {
    fn from_bits(value: impl Bits) -> Self {
        Offset(value.full() as u16)
    }
}

impl<R: Reader> ParseInstruction<R> for MUBUFInstruction {
    fn parse(token: u32, reader: &mut R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MUBUFInstruction::from_bits(token))
    }
}

impl DisplayInstruction for MUBUFInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: {
                let mut args = vec![
                    self.vdata.display(&Some(OperandInfo::Size(match self.op {
                        MUBUFOpCode::buffer_load_format_x => 1,
                        MUBUFOpCode::buffer_load_format_xy => 2,
                        MUBUFOpCode::buffer_load_format_xyz => 3,
                        MUBUFOpCode::buffer_load_format_xyzw => 4,
                        MUBUFOpCode::buffer_store_format_x => 1,
                        MUBUFOpCode::buffer_store_format_xy => 2,
                        MUBUFOpCode::buffer_store_format_xyz => 3,
                        MUBUFOpCode::buffer_store_format_xyzw => 4,
                        _ => unimplemented!(),
                    }))),
                    self.vaddr.display(&None),
                    self.srsrc.display(),
                ];

                if self.offen {
                    args.push("offen".to_string());
                }

                if self.idxen {
                    args.push("idxen".to_string());
                }

                if self.glc {
                    args.push("glc".to_string());
                }

                if self.addr64 {
                    args.push("addr64".to_string());
                }

                if self.lds {
                    args.push("lds".to_string());
                }

                if self.slc {
                    args.push("slc".to_string());
                }

                if self.tfe {
                    args.push("tfe".to_string());
                }

                args.push(format!("offset=0x{:x}", self.offset.0));
                args.push(format!("soffset=0x{:x}", self.soffset));

                args
            },
        }
    }
}
