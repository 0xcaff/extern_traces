use crate::bitrange::bitrange;
use crate::instructions::formats::{combine, ParseInstruction, Reader};
use crate::instructions::generated::MIMGOpCode;

#[derive(Debug)]
pub struct MIMGInstruction {
    op: MIMGOpCode,
    // todo: remaining fields
}

impl<R: Reader> ParseInstruction<R> for MIMGInstruction {
    fn parse(token: u32, reader: R) -> Result<Self, anyhow::Error> {
        let token = combine(token, reader)?;
        Ok(MIMGInstruction {
            op: MIMGOpCode::decode(bitrange(7, 7).of_64(token))?,
        })
    }
}
