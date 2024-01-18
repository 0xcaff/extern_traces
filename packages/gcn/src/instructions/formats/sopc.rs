use crate::instructions::display::{DisplayInstruction, DisplayableInstruction};
use crate::instructions::generated::SOPCOpCode;
use crate::instructions::operands::ScalarSourceOperand;
use bits_macros::FromBits;

/// Scalar Instruction Two Inputs, One Comparison
///
/// Scalar instruction taking two inputs and producing a comparison result. Can
/// be followed by a 32-bit literal constant
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPCInstruction {
    #[bits(22, 16)]
    op: SOPCOpCode,

    #[bits(7, 0)]
    ssrc0: ScalarSourceOperand,

    #[bits(15, 8)]
    ssrc1: ScalarSourceOperand,
}

impl DisplayInstruction for SOPCInstruction {
    fn display(&self) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.ssrc0.display(&op_info.operands[0]),
                self.ssrc1.display(&op_info.operands[1]),
            ],
        }
    }
}
