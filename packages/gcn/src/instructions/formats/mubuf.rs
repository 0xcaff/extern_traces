use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MUBUFOpCode;
use bits::{bitrange, FromBits};

#[derive(Debug)]
pub struct MUBUFInstruction {
    op: MUBUFOpCode,
}

impl<R: Reader> ParseInstruction<R> for MUBUFInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MUBUFInstruction {
            op: MUBUFOpCode::from_bits(bitrange(7, 7).of_64(token)),
        })
    }
}
