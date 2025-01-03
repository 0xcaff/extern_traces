use crate::instructions::display::DisplayInstruction;
use crate::instructions::operands::VectorGPR;
use crate::instructions::ops::VINTRPOpCode;
use crate::instructions::DisplayableInstruction;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use bits::{Bits, FromBits, TryFromBits};
use bits_macros::TryFromBitsContainer;
use strum::{AsRefStr, FromRepr};

#[derive(Debug, TryFromBitsContainer)]
#[bits(32)]
pub struct VINTRPInstruction {
    #[bits(17, 16)]
    pub op: VINTRPOpCode,

    #[bits(7, 0)]
    pub vsrc: VectorGPR,

    #[bits(9, 8)]
    pub attribute_channel: AttributeChannel,

    #[bits(15, 10)]
    pub attr: Attr,

    #[bits(25, 18)]
    pub vdst: VectorGPR,
}

#[derive(FromRepr, Debug, AsRefStr, Copy, Clone)]
#[repr(u8)]
pub enum AttributeChannel {
    #[strum(serialize = "x")]
    X = 0,
    #[strum(serialize = "y")]
    Y = 1,
    #[strum(serialize = "z")]
    Z = 2,
    #[strum(serialize = "w")]
    W = 3,
}

impl TryFromBits<2> for AttributeChannel {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        Self::from_repr(value.full() as u8)
    }
}

#[derive(Debug)]
pub struct Attr(pub u8);

impl FromBits<6> for Attr {
    fn from_bits(value: impl Bits) -> Self {
        Self(value.full() as _)
    }
}

impl DisplayInstruction for VINTRPInstruction {
    fn display(&self, _: Option<u32>) -> DisplayableInstruction {
        let op_info = self.op.instruction_info();

        DisplayableInstruction {
            op: self.op.as_ref().to_string(),
            args: vec![
                self.vdst.display(&op_info.definitions[0]),
                self.vsrc.display(&op_info.operands[0]),
                format!("attr{}.{}", self.attr.0, self.attribute_channel.as_ref()),
            ],
        }
    }
}
