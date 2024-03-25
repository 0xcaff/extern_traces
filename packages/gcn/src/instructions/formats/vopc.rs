use crate::instructions::generated::VOPCOpCode;
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::{DisplayInstruction, DisplayableInstruction};
use bits_macros::FromBits;

#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VOPCInstruction {
    #[bits(24, 17)]
    pub op: VOPCOpCode,

    #[bits(8, 0)]
    pub src0: SourceOperand,

    #[bits(16, 9)]
    pub vsrc1: VectorGPR,
}

impl DisplayInstruction for VOPCInstruction {
    fn display(&self) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.src0.display(&op_info.operands[0]),
                self.vsrc1.display(&op_info.operands[1]),
            ],
        }
    }
}
