use crate::instructions::display::DisplayInstruction;
use crate::instructions::formats::{combine, ParseInstruction};
use crate::instructions::operands::VectorGPR;
use crate::instructions::ops::DSOpCode;
use crate::instructions::DisplayableInstruction;
use crate::SliceReader;
use alloc::string::ToString;
use alloc::vec;
use bits::TryFromBitsContainer;
use bits_macros::TryFromBitsContainer;

/// Data Share Instruction
#[derive(Debug, TryFromBitsContainer)]
#[bits(64)]
pub struct DSInstruction {
    #[bits(25, 18)]
    pub op: DSOpCode,

    #[bits(7, 0)]
    pub offset0: u8,

    #[bits(15, 8)]
    pub offset1: u8,

    #[bits(17, 17)]
    pub gds: bool,

    #[bits(39, 32)]
    pub addr: VectorGPR,

    #[bits(47, 40)]
    pub data0: VectorGPR,

    #[bits(55, 48)]
    pub data1: VectorGPR,

    #[bits(63, 56)]
    pub vdst: VectorGPR,
}

impl ParseInstruction for DSInstruction {
    fn parse(token: u32, reader: &mut SliceReader) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;

        Ok(DSInstruction::try_from_bits_container(token)?)
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
