use crate::instructions::display::DisplayInstruction;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use crate::instructions::ops::SOP1OpCode;
use crate::instructions::DisplayableInstruction;
use alloc::string::ToString;
use alloc::vec;
use bits_macros::TryFromBitsContainer;

/// Scalar Instruction One Input, One Output
///
/// This is a scalar instruction with one input and one output. Can be followed
/// by a 32-bit literal constant.
#[derive(Debug, TryFromBitsContainer)]
#[bits(32)]
pub struct SOP1Instruction {
    #[bits(15, 8)]
    pub op: SOP1OpCode,

    #[bits(7, 0)]
    pub ssrc0: ScalarSourceOperand,

    #[bits(22, 16)]
    pub sdst: ScalarDestinationOperand,
}

impl DisplayInstruction for SOP1Instruction {
    fn display(&self, literal_constant: Option<u32>) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.sdst.display(&op_info.definitions[0]),
                self.ssrc0.display(&op_info.operands[0], literal_constant),
            ],
        }
    }
}
