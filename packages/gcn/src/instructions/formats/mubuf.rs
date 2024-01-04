use crate::bitrange::bitrange;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MUBUFOpCode;
use crate::reader::ReadError;

#[derive(Debug)]
pub struct MUBUFInstruction {
    op: MUBUFOpCode,
}

impl<R: Reader> ParseInstruction<R> for MUBUFInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, ReadError> {
        let token = combine(token, reader)?;
        Ok(MUBUFInstruction {
            op: MUBUFOpCode::decode(bitrange(7, 7).of_64(token))?,
        })
    }
}
