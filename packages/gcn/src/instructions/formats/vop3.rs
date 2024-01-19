use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::{VOP1OpCode, VOP2OpCode, VOP3OpCode, VOPCOpCode};
use crate::instructions::instruction_info::{InstructionInfo, OperandInfo};
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::{DisplayInstruction, DisplayableInstruction};
use anyhow::format_err;
use bits::{bitrange, FromBits};
use bits_macros::FromBits;
use strum::FromRepr;

#[derive(Debug, FromBits)]
#[bits(64)]
pub struct VOP3Instruction {
    #[bits(25, 17)]
    op: OpCode,

    #[bits(7, 0)]
    vdst: VectorGPR,

    #[bits(src(0))]
    src0: TransformedOperand,

    #[bits(src(1))]
    src1: TransformedOperand,

    #[bits(src(2))]
    src2: TransformedOperand,

    #[bits(11, 11)]
    clamp: bool,

    #[bits(60, 59)]
    output_modifier: OutputModifier,
}

#[derive(Debug, FromRepr)]
#[repr(usize)]
enum OutputModifier {
    None = 0,
    Mul2 = 1,
    Mul4 = 2,
    Div2 = 3,
}

impl FromBits<2> for OutputModifier {
    fn from_bits(value: usize) -> Self {
        Self::from_repr(value).unwrap()
    }
}

impl OutputModifier {
    fn display(&self) -> Option<String> {
        match self {
            OutputModifier::None => None,
            OutputModifier::Mul2 => Some("mul:2".to_string()),
            OutputModifier::Mul4 => Some("mul:4".to_string()),
            OutputModifier::Div2 => Some("div:2".to_string()),
        }
    }
}

impl<R: Reader> ParseInstruction<R> for VOP3Instruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(VOP3Instruction::from_bits(token as usize))
    }
}

fn src(idx: u8) -> impl Fn(usize) -> TransformedOperand {
    move |token| TransformedOperand::parse(token as _, idx)
}

#[derive(Debug)]
pub enum OpCode {
    VOP3(VOP3OpCode),
    VOPC(VOPCOpCode),
    VOP2(VOP2OpCode),
    VOP1(VOP1OpCode),
}

impl OpCode {
    fn from(op: usize) -> Option<OpCode> {
        if let Some(value) = VOPCOpCode::from_repr(op) {
            return Some(OpCode::VOPC(value));
        };

        if let Some(value) = VOP3OpCode::from_repr(op) {
            return Some(OpCode::VOP3(value));
        };

        if let Some(value) = VOP2OpCode::from_repr(op - 256) {
            return Some(OpCode::VOP2(value));
        };

        if let Some(value) = VOP1OpCode::from_repr(op - 384) {
            return Some(OpCode::VOP1(value));
        };

        None
    }

    fn decode(op: usize) -> Result<OpCode, anyhow::Error> {
        Ok(Self::from(op).ok_or_else(|| format_err!("unknown op code {} for VOP3", op))?)
    }

    fn instruction_info(&self) -> InstructionInfo {
        match self {
            OpCode::VOP3(op) => op.instruction_info(),
            OpCode::VOPC(op) => op.instruction_info(),
            OpCode::VOP2(op) => op.instruction_info(),
            OpCode::VOP1(op) => op.instruction_info(),
        }
    }
}

impl AsRef<str> for OpCode {
    fn as_ref(&self) -> &str {
        match self {
            OpCode::VOP3(op) => op.as_ref(),
            OpCode::VOPC(op) => op.as_ref(),
            OpCode::VOP2(op) => op.as_ref(),
            OpCode::VOP1(op) => op.as_ref(),
        }
    }
}

impl FromBits<9> for OpCode {
    fn from_bits(value: usize) -> Self {
        Self::decode(value).unwrap()
    }
}

#[derive(Debug)]
struct TransformedOperand {
    operand: SourceOperand,
    abs: bool,
    neg: bool,
}

impl TransformedOperand {
    fn parse(token: u64, idx: u8) -> TransformedOperand {
        let highest_idx = match idx {
            0 => 40,
            1 => 49,
            2 => 58,
            _ => panic!("invalid index {}", idx),
        };

        let op_value = bitrange(highest_idx, highest_idx - 8).of(token as _);

        let abs_idx = 10 - idx;
        let neg_idx = 63 - idx;

        TransformedOperand {
            abs: bitrange(abs_idx, abs_idx).of(token as _) == 1,
            neg: bitrange(neg_idx, neg_idx).of(token as _) == 1,
            operand: SourceOperand::from_bits(op_value),
        }
    }

    fn display(&self, operand_info: &Option<OperandInfo>) -> String {
        let result = format!("{}", self.operand.display(operand_info));

        let result = if self.abs {
            format!("abs({})", result)
        } else {
            result
        };

        let result = if self.neg {
            format!("-{}", result)
        } else {
            result
        };

        result
    }
}

impl DisplayInstruction for VOP3Instruction {
    fn display(&self) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        // todo: argument count
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: {
                let mut args = vec![
                    self.vdst.display(&op_info.definitions[0]),
                    self.src0.display(&op_info.operands[0]),
                    self.src1.display(&op_info.operands[1]),
                    self.src2.display(&op_info.operands[2]),
                ];

                if self.clamp {
                    args.push("clamp".to_string());
                }

                if let Some(value) = self.output_modifier.display() {
                    args.push(value)
                }

                args
            },
        }
    }
}
