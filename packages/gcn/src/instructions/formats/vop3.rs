use crate::format_pattern;
use crate::instructions::format_info::{bitrange, OpFormatPattern};
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::{VOP1OpCode, VOP2OpCode, VOP3OpCode, VOPCOpCode};
use crate::instructions::operands::{SourceOperand, VectorGPR};
use crate::instructions::InstructionParseErrorKind;
use anyhow::format_err;

pub const FORMAT: OpFormatPattern = format_pattern!("110100");

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
    fn parse(first_token: u32, mut reader: R) -> Result<Self, InstructionParseErrorKind> {
        let second_token = reader.read_u32()?;

        let token = ((first_token as u64) << 32) | second_token;

        Ok(VOP3Instruction {
            op: OpCode::decode(bitrange(6, 9).of(token) as u16)?,
            vdst: VectorGPR::decode(bitrange(24, 8).of(token) as u8),

            src0: TransformedOperand::parse(token, 0),
            src1: TransformedOperand::parse(token, 1),
            src2: TransformedOperand::parse(token, 2),

            clamp: bitrange(20, 1).of(token) == 1,
        })
    }
}

pub enum OpCode {
    VOP3(VOP3OpCode),
    VOPC(VOPCOpCode),
    VOP2(VOP2OpCode),
    VOP1(VOP1OpCode),
}

impl OpCode {
    fn from(op: u16) -> Option<OpCode> {
        match VOPCOpCode::from_repr(op) {
            Some(value) => return Some(OpCode::VOPC(value)),
            None => {}
        };

        match VOP3OpCode::from_repr(op) {
            Some(value) => return Some(OpCode::VOP3(value)),
            None => {}
        };

        match VOP2OpCode::from_repr(op - 256) {
            Some(value) => return Some(OpCode::VOP2(value)),
            None => {}
        };

        match VOP1OpCode::from_repr(op - 384) {
            Some(value) => return Some(OpCode::VOP1(value)),
            None => {}
        }

        None
    }

    fn parse(op: u16) -> Result<OpCode, anyhow::Error> {
        Ok(Self::from(op).ok_or_else(|| format_err!("unknown op code {} for VOP3", op))?)
    }
}

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
        .of(token) as u16;

        TransformedOperand {
            abs: bitrange(21 + idx, 1).of(token) == 1,
            neg: bitrange(32 + idx, 1).of(token) == 1,
            operand: SourceOperand::decode(op_value),
        }
    }
}
