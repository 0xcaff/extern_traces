use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::{VOP1OpCode, VOP2OpCode, VOP3OpCode, VOPCOpCode};
use crate::instructions::operands::{SourceOperand, VectorGPR};
use anyhow::format_err;
use bits::{bitrange, FromBits};

#[derive(Debug)]
pub struct VOP3Instruction {
    op: OpCode,

    vdst: VectorGPR,

    src0: TransformedOperand,
    src1: TransformedOperand,
    src2: TransformedOperand,

    clamp: bool,
    // todo: omod
}

impl<R: Reader> ParseInstruction<R> for VOP3Instruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(VOP3Instruction {
            op: OpCode::decode(bitrange(25, 17).of(token as _))?,
            vdst: VectorGPR::from_bits(bitrange(7, 0).of(token as _)),

            src0: TransformedOperand::parse(token, 0),
            src1: TransformedOperand::parse(token, 1),
            src2: TransformedOperand::parse(token, 2),

            clamp: bitrange(11, 11).of(token as _) == 1,
        })
    }
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
