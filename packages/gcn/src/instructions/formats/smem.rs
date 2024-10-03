use crate::instructions::display::DisplayInstruction;
use crate::instructions::instruction_info::OperandInfo;
use crate::instructions::operands::ScalarDestinationOperand;
use crate::instructions::ops::SMEMOpCode;
use crate::instructions::DisplayableInstruction;
use alloc::string::ToString;
use alloc::{format, vec};
use bits_macros::TryFromBitsContainer;

/// Scalar Instruction Memory Access
///
/// Scalar instruction performing a memory operation on scalar L1 memory. Two
/// Dwords.
#[derive(Debug, TryFromBitsContainer)]
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
                self.sdst.display(&Some(OperandInfo::Size(match self.op {
                    SMEMOpCode::s_load_dword => 1,
                    SMEMOpCode::s_load_dwordx2 => 2,
                    SMEMOpCode::s_load_dwordx4 => 4,
                    SMEMOpCode::s_load_dwordx8 => 8,
                    SMEMOpCode::s_load_dwordx16 => 16,
                    SMEMOpCode::s_buffer_load_dword => 1,
                    SMEMOpCode::s_buffer_load_dwordx2 => 2,
                    SMEMOpCode::s_buffer_load_dwordx4 => 4,
                    SMEMOpCode::s_buffer_load_dwordx8 => 8,
                    SMEMOpCode::s_buffer_load_dwordx16 => 16,
                    _ => unimplemented!(),
                }))),
                ScalarDestinationOperand::ScalarGPR((self.sbase << 1) as u8).display(&Some(
                    OperandInfo::Size(match self.op {
                        SMEMOpCode::s_load_dword
                        | SMEMOpCode::s_load_dwordx2
                        | SMEMOpCode::s_load_dwordx4
                        | SMEMOpCode::s_load_dwordx8
                        | SMEMOpCode::s_load_dwordx16 => 2,
                        SMEMOpCode::s_buffer_load_dword
                        | SMEMOpCode::s_buffer_load_dwordx2
                        | SMEMOpCode::s_buffer_load_dwordx4
                        | SMEMOpCode::s_buffer_load_dwordx8
                        | SMEMOpCode::s_buffer_load_dwordx16 => 4,
                        _ => unimplemented!(),
                    }),
                )),
                format!("{:#x}", self.offset),
            ],
        }
    }
}
