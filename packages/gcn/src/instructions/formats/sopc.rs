use alloc::string::ToString;
use alloc::vec;
use crate::instructions::display::{DisplayInstruction, DisplayableInstruction};
use crate::instructions::operands::ScalarSourceOperand;
use crate::instructions::ops::SOPCOpCode;
use bits_macros::FromBits;

/// Scalar Instruction Two Inputs, One Comparison
///
/// Scalar instruction taking two inputs and producing a comparison result. Can
/// be followed by a 32-bit literal constant
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPCInstruction {
    #[bits(22, 16)]
    pub op: SOPCOpCode,

    #[bits(7, 0)]
    pub ssrc0: ScalarSourceOperand,

    #[bits(15, 8)]
    pub ssrc1: ScalarSourceOperand,
}

impl DisplayInstruction for SOPCInstruction {
    fn display(&self, literal_constant: Option<u32>) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.ssrc0.display(&op_info.operands[0], literal_constant),
                self.ssrc1.display(&op_info.operands[1], literal_constant),
            ],
        }
    }
}
