use crate::instructions::display::DisplayInstruction;
use crate::instructions::ops::SOPPOpCode;
use crate::instructions::DisplayableInstruction;
use bits_macros::FromBits;

/// Scalar Instruction One Input, One Special Operation
///
/// Scalar instruction taking one inline constant input and performing a
/// special operation (for example: jump).
#[derive(Debug, FromBits)]
#[bits(32)]
pub struct SOPPInstruction {
    #[bits(22, 16)]
    pub op: SOPPOpCode,

    #[bits(15, 0)]
    pub simm16: i16,
}

impl DisplayInstruction for SOPPInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![if self.simm16 < 0 {
                format!("-0x{:x}", self.simm16.abs())
            } else {
                format!("0x{:x}", self.simm16)
            }],
        }
    }
}
