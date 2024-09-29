use crate::instructions::display::DisplayInstruction;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::instructions::ops::VOP1OpCode;
use crate::instructions::DisplayableInstruction;
use alloc::string::ToString;
use alloc::vec;
use bits_macros::TryFromBitsContainer;

/// Vector Instruction One Input, One Output
///
/// Vector instruction taking one input and producing one output. Can be
/// followed by a 32-bit literal constant.
#[derive(Debug, TryFromBitsContainer)]
#[bits(32)]
pub struct VOP1Instruction {
    #[bits(16, 9)]
    pub op: VOP1OpCode,

    #[bits(8, 0)]
    pub src0: SourceOperand,

    #[bits(24, 17)]
    pub vdst: VectorGPR,
}

impl DisplayInstruction for VOP1Instruction {
    fn display(&self, literal_constant: Option<u32>) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.vdst.display(&op_info.definitions[0]),
                self.src0.display(&op_info.operands[0], literal_constant),
            ],
        }
    }
}
