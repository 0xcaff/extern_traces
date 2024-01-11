use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::VINTRPOpCode;
use bits::bitrange;

#[derive(Debug)]
pub struct VINTRPInstruction {
    op: VINTRPOpCode,
    // todo: implement
}

impl<R: Reader> ParseInstruction<R> for VINTRPInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(VINTRPInstruction {
            op: VINTRPOpCode::decode(bitrange(14, 2).of_32(token))?,
        })
    }
}
