use crate::instructions::generated::VOP1OpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::{DisplayInstruction, DisplayableInstruction};
use bits_macros::FromBits;

/// Vector Instruction One Input, One Output
///
/// Vector instruction taking one input and producing one output. Can be
/// followed by a 32-bit literal constant.
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VOP1Instruction {
    #[bits(16, 9)]
    op: VOP1OpCode,

    #[bits(8, 0)]
    src0: SourceOperand,

    #[bits(24, 17)]
    vdst: VectorGPR,
}

impl DisplayInstruction for VOP1Instruction {
    fn display(&self) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.vdst.display(&op_info.definitions[0]),
                self.src0.display(&op_info.operands[0]),
            ],
        }
    }
}
