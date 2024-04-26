use crate::instructions::display::DisplayInstruction;
use crate::instructions::formats::{combine, ParseInstruction};
use crate::instructions::operands::VectorGPR;
use crate::instructions::ops::DSOpCode;
use crate::instructions::DisplayableInstruction;
use crate::reader::Reader;
use bits::FromBits;
use bits_macros::FromBits;

/// Data Share Instruction
#[derive(Debug, FromBits)]
#[bits(64)]
pub struct DSInstruction {
    #[bits(25, 18)]
    op: DSOpCode,

    #[bits(7, 0)]
    offset0: u8,

    #[bits(15, 8)]
    offset1: u8,

    #[bits(17, 17)]
    gds: bool,

    #[bits(39, 32)]
    addr: VectorGPR,

    #[bits(47, 40)]
    data0: VectorGPR,

    #[bits(55, 48)]
    data1: VectorGPR,

    #[bits(63, 56)]
    vdst: VectorGPR,
}

impl<R: Reader> ParseInstruction<R> for DSInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;

        Ok(DSInstruction::from_bits(token))
    }
}

impl DisplayInstruction for DSInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        // todo: implement
        DisplayableInstruction {
            op: "unknown".to_string(),
            args: vec![],
        }
    }
}
