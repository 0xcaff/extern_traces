use crate::bitrange::bitrange;
use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SMEMOpCode;
use crate::reader::ReadError;

/// Scalar Instruction Memory Access
///
/// Scalar instruction performing a memory operation on scalar L1 memory. Two
/// Dwords.
#[derive(Debug)]
pub struct SMEMInstruction {
    op: SMEMOpCode,
    // todo: implement
}

impl<R: Reader> ParseInstruction<R> for SMEMInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, ReadError> {
        Ok(SMEMInstruction {
            op: SMEMOpCode::decode(bitrange(5, 5).of_32(token))?,
        })
    }
}
