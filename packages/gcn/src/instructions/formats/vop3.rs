use crate::bitrange::bitrange;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::{VOP1OpCode, VOP2OpCode, VOP3OpCode, VOPCOpCode};
use crate::instructions::operands::{SourceOperand, VectorGPR};
use anyhow::format_err;

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
            op: OpCode::decode(bitrange(6, 9).of_64(token))?,
            vdst: VectorGPR::decode(bitrange(24, 8).of_64(token) as u8),

            src0: TransformedOperand::parse(token, 0),
            src1: TransformedOperand::parse(token, 1),
            src2: TransformedOperand::parse(token, 2),

            clamp: bitrange(20, 1).of_64(token) == 1,
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
        let op_value = bitrange(
            match idx {
                2 => 37,
                1 => 46,
                0 => 55,
                _ => panic!("invalid index {}", idx),
            },
            9,
        )
        .of_64(token) as u16;

        TransformedOperand {
            abs: bitrange(21 + idx, 1).of_64(token) == 1,
            neg: bitrange(32 + idx, 1).of_64(token) == 1,
            operand: SourceOperand::decode(op_value),
        }
    }
}
