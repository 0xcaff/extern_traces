use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::{VOP1OpCode, VOP2OpCode, VOP3OpCode, VOPCOpCode};
use crate::instructions::operands::{SourceOperand, VectorGPR};
use anyhow::format_err;
use bits::{bitrange, FromBits};
use bits_macros::FromBits;
use crate::{DisplayableInstruction, DisplayInstruction};

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
    // todo: omod
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
}

impl DisplayInstruction for VOP3Instruction {
    fn display(&self) -> DisplayableInstruction {
        // todo: implement
        DisplayableInstruction {
            op: "unknown".to_string(),
            args: vec![],
        }
    }
}
