use crate::instructions::formats::ParseInstruction;
use crate::instructions::generated::SOPCOpCode;
use crate::instructions::operands::ScalarSourceOperand;
use crate::reader::Reader;
use bits::FromBits;
use bits_macros::FromBits;

/// Scalar Instruction Two Inputs, One Comparison
///
/// Scalar instruction taking two inputs and producing a comparison result. Can
/// be followed by a 32-bit literal constant
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPCInstruction {
    #[bits(16, 22)]
    op: SOPCOpCode,

    #[bits(0, 7)]
    ssrc0: ScalarSourceOperand,

    #[bits(8, 15)]
    ssrc1: ScalarSourceOperand,
}

impl<R: Reader> ParseInstruction<R> for SOPCInstruction {
    fn parse(token: u32, _reader: R) -> Result<Self, anyhow::Error> {
        Ok(Self::from_bits(token as usize))
    }
}
