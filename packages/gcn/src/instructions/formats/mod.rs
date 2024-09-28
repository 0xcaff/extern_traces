pub use crate::instructions::formats::ds::DSInstruction;
pub use crate::instructions::formats::exp::ExpInstruction;
pub use crate::instructions::formats::mimg::MIMGInstruction;
pub use crate::instructions::formats::mtbuf::MTBufInstruction;
pub use crate::instructions::formats::mubuf::MUBUFInstruction;
pub use crate::instructions::formats::smem::SMEMInstruction;
pub use crate::instructions::formats::sop1::SOP1Instruction;
pub use crate::instructions::formats::sop2::SOP2Instruction;
pub use crate::instructions::formats::sopc::SOPCInstruction;
pub use crate::instructions::formats::sopk::SOPKInstruction;
pub use crate::instructions::formats::sopp::SOPPInstruction;
pub use crate::instructions::formats::vintrp::VINTRPInstruction;
pub use crate::instructions::formats::vop1::VOP1Instruction;
pub use crate::instructions::formats::vop2::VOP2Instruction;
pub use crate::instructions::formats::vop3::VOP3Instruction;
pub use crate::instructions::formats::vopc::VOPCInstruction;
use crate::instructions::operands::{ScalarSourceOperand, SourceOperand};
use crate::instructions::ops::{SOPKOpCode, VOP1OpCode, VOP2OpCode};
use crate::reader::EofError;
use crate::SliceReader;
use bits::FromBits;
use gcn_internal_macros::{DisplayInstruction, ParseInstruction};

pub mod ds;
pub mod exp;
pub mod mimg;
pub mod mtbuf;
pub mod mubuf;
pub mod smem;
pub mod sop1;
pub mod sop2;
pub mod sopc;
pub mod sopk;
pub mod sopp;
pub mod vintrp;
pub mod vop1;
pub mod vop2;
pub mod vop3;
pub mod vopc;

#[derive(Debug, ParseInstruction, DisplayInstruction)]
pub enum FormattedInstruction {
    #[pattern(0b110110)]
    DS(DSInstruction),

    #[pattern(0b111110)]
    EXP(ExpInstruction),

    #[pattern(0b111100)]
    MIMG(MIMGInstruction),

    #[pattern(0b111010)]
    MTBUF(MTBufInstruction),

    #[pattern(0b111000)]
    MUBUF(MUBUFInstruction),

    #[pattern(0b11000)]
    SMEM(SMEMInstruction),

    #[pattern(0b101111101)]
    SOP1(SOP1Instruction),

    #[pattern(0b10)]
    SOP2(SOP2Instruction),

    #[pattern(0b101111100)]
    SOPC(SOPCInstruction),

    #[pattern(0b1011)]
    SOPK(SOPKInstruction),

    #[pattern(0b101111111)]
    SOPP(SOPPInstruction),

    #[pattern(0b110010)]
    VINTRP(VINTRPInstruction),

    #[pattern(0b0111111)]
    VOP1(VOP1Instruction),

    #[pattern(0b0)]
    VOP2(VOP2Instruction),

    #[pattern(0b110100)]
    VOP3(VOP3Instruction),

    #[pattern(0b0111110)]
    VOPC(VOPCInstruction),
}

impl FormattedInstruction {
    pub fn has_literal_constant(&self) -> bool {
        matches!(
            self,
            FormattedInstruction::SOP2(
                SOP2Instruction {
                    ssrc0: ScalarSourceOperand::LiteralConstant,
                    ..
                } | SOP2Instruction {
                    ssrc1: ScalarSourceOperand::LiteralConstant,
                    ..
                },
            ) | FormattedInstruction::SOPK(SOPKInstruction {
                op: SOPKOpCode::s_setreg_imm32_b32,
                ..
            }) | FormattedInstruction::SOP1(SOP1Instruction {
                ssrc0: ScalarSourceOperand::LiteralConstant,
                ..
            }) | FormattedInstruction::SOPC(
                SOPCInstruction {
                    ssrc0: ScalarSourceOperand::LiteralConstant,
                    ..
                } | SOPCInstruction {
                    ssrc1: ScalarSourceOperand::LiteralConstant,
                    ..
                },
            ) | FormattedInstruction::VOP1(VOP1Instruction {
                src0: SourceOperand::Scalar(ScalarSourceOperand::LiteralConstant),
                ..
            }) | FormattedInstruction::VOP2(
                VOP2Instruction {
                    src0: SourceOperand::Scalar(ScalarSourceOperand::LiteralConstant),
                    ..
                } | VOP2Instruction {
                    op: VOP2OpCode::v_madmk_f32 | VOP2OpCode::v_madak_f32,
                    ..
                },
            ) | FormattedInstruction::VOPC(VOPCInstruction {
                src0: SourceOperand::Scalar(ScalarSourceOperand::LiteralConstant),
                ..
            })
        )
    }

    /// Whether this instruction is disabled if the thread's exec bit is 0. Instructions like
    /// v_cmp_gt_u32 which are modified by the exec mask but not disabled by it will not be
    /// included.
    pub fn exec_mask_disables(&self) -> bool {
        fn vop3(vop3: &vop3::OpCode) -> bool {
            match vop3 {
                vop3::OpCode::VOP3(_) => {
                    unimplemented!()
                }
                vop3::OpCode::VOPC(..) => false,
                vop3::OpCode::VOP2(op) => match op {
                    VOP2OpCode::v_readlane_b32 | VOP2OpCode::v_writelane_b32 => false,
                    _ => true,
                },
                vop3::OpCode::VOP1(op) => match op {
                    VOP1OpCode::v_readfirstlane_b32 => false,
                    _ => true,
                },
            }
        }

        match self {
            // Vector ALU
            FormattedInstruction::VOP2(VOP2Instruction { op, .. }) => {
                vop3(&vop3::OpCode::VOP2(*op))
            }
            FormattedInstruction::VOP1(VOP1Instruction { op, .. }) => {
                vop3(&vop3::OpCode::VOP1(*op))
            }
            FormattedInstruction::VOP3(VOP3Instruction { op, .. }) => vop3(op),
            FormattedInstruction::VINTRP(..) => true,

            // Vector Memory
            FormattedInstruction::MUBUF(..)
            | FormattedInstruction::MTBUF(..)
            | FormattedInstruction::MIMG(..) => true,

            // Local and Global Data Share
            FormattedInstruction::DS(..) => true,

            // Export
            FormattedInstruction::EXP(..) => true,

            // Scalar Operations (starting with s_)
            FormattedInstruction::SMEM(..)
            | FormattedInstruction::SOP1(..)
            | FormattedInstruction::SOP2(..)
            | FormattedInstruction::SOPC(..)
            | FormattedInstruction::SOPK(..)
            | FormattedInstruction::SOPP(..) => false,

            // VOPC is handled at instruction level
            FormattedInstruction::VOPC(..) => false,
        }
    }
}

pub trait ParseInstruction {
    fn parse(token: u32, reader: &mut SliceReader) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

impl<T: FromBits<32>> ParseInstruction for T {
    fn parse(token: u32, _reader: &mut SliceReader) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Self::from_bits(token))
    }
}

fn combine(first_token: u32, reader: &mut SliceReader) -> Result<u64, EofError> {
    let second_token = reader.read_u32()?;
    let token = (first_token as u64) | ((second_token as u64) << 32);

    Ok(token)
}
