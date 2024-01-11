use crate::instructions::formats::{ParseInstruction, Reader};
use crate::instructions::generated::SOPCOpCode;
use crate::instructions::operands::ScalarSourceOperand;
use bits::{bitrange, FromBits};

/// Scalar Instruction Two Inputs, One Comparison
///
/// Scalar instruction taking two inputs and producing a comparison result. Can
/// be followed by a 32-bit literal constant
#[derive(Debug)]
pub struct SOPCInstruction {
    op: SOPCOpCode,

    ssrc0: ScalarSourceOperand,
    ssrc1: ScalarSourceOperand,
}

impl<R: Reader> ParseInstruction<R> for SOPCInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(SOPCInstruction {
            op: SOPCOpCode::decode(bitrange(9, 7).of_32(token))?,
            ssrc0: ScalarSourceOperand::from_bits(bitrange(16, 8).of_32(token)),
            ssrc1: ScalarSourceOperand::from_bits(bitrange(24, 8).of_32(token)),
        })
    }
}
