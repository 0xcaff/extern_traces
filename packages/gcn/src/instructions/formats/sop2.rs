use crate::instructions::generated::SOP2OpCode;
use crate::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use crate::{DisplayInstruction, DisplayableInstruction};
use bits_macros::FromBits;

/// Scalar Format Two Inputs, One Output
///
/// This is a scalar instruction with two inputs and one output. Can be
/// followed by a 32-bit literal
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOP2Instruction {
    #[bits(29, 23)]
    pub op: SOP2OpCode,

    #[bits(7, 0)]
    pub ssrc0: ScalarSourceOperand,

    #[bits(15, 8)]
    pub ssrc1: ScalarSourceOperand,

    #[bits(22, 16)]
    pub sdst: ScalarDestinationOperand,
}

impl DisplayInstruction for SOP2Instruction {
    fn display(&self) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.sdst.display(&op_info.definitions[0]),
                self.ssrc0.display(&op_info.operands[0]),
                self.ssrc1.display(&op_info.operands[1]),
            ],
        }
    }
}
