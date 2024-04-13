use crate::instructions::generated::SMEMOpCode;
use crate::instructions::instruction_info::OperandInfo;
use crate::instructions::operands::ScalarDestinationOperand;
use crate::{DisplayInstruction, DisplayableInstruction};
use bits_macros::FromBits;

/// Scalar Instruction Memory Access
///
/// Scalar instruction performing a memory operation on scalar L1 memory. Two
/// Dwords.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SMEMInstruction {
    #[bits(26, 22)]
    pub op: SMEMOpCode,

    #[bits(7, 0)]
    pub offset: u8,

    #[bits(8, 8)]
    pub imm: bool,

    #[bits(14, 9)]
    pub sbase: u64,

    #[bits(21, 15)]
    pub sdst: ScalarDestinationOperand,
}

impl DisplayInstruction for SMEMInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                // todo: this isn't correct generally
                self.sdst.display(&Some(OperandInfo::Size(4))),
                ScalarDestinationOperand::ScalarGPR((self.sbase << 1) as u8)
                    .display(&Some(OperandInfo::Size(2))),
                format!("0x{:x}", self.offset),
            ],
        }
    }
}
