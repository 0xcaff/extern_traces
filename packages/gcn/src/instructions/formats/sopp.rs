use crate::instructions::display::DisplayInstruction;
use crate::instructions::ops::SOPPOpCode;
use crate::instructions::DisplayableInstruction;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use bits_macros::TryFromBitsContainer;

/// Scalar Instruction One Input, One Special Operation
///
/// Scalar instruction taking one inline constant input and performing a
/// special operation (for example: jump).
#[derive(Debug, TryFromBitsContainer)]
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
                format!("-{:#x}", self.simm16.abs())
            } else {
                format!("{:#x}", self.simm16)
            }],
        }
    }
}
