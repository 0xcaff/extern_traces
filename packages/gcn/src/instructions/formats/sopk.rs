use crate::instructions::generated::SOPKOpCode;
use crate::instructions::operands::ScalarDestinationOperand;
use crate::{DisplayInstruction, DisplayableInstruction};
use bits_macros::FromBits;

/// Scalar Instruction One Inline Constant Input, One Output
///
/// This is a scalar instruction with one inline constant input and one output.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPKInstruction {
    #[bits(27, 23)]
    pub op: SOPKOpCode,

    #[bits(15, 0)]
    pub simm16: u16,

    #[bits(22, 16)]
    pub sdst: ScalarDestinationOperand,
}

impl DisplayInstruction for SOPKInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.sdst.display(&op_info.definitions[0]),
                format!("0x{:x}", self.simm16),
            ],
        }
    }
}
