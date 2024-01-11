use crate::instructions::generated::SOPPOpCode;
use bits_macros::FromBits;

/// Scalar Instruction One Input, One Special Operation
///
/// Scalar instruction taking one inline constant input and performing a
/// special operation (for example: jump).
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPPInstruction {
    #[bits(22, 16)]
    op: SOPPOpCode,

    #[bits(15, 0)]
    simm16: u16,
}
