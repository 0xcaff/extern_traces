use crate::instructions::generated::VINTRPOpCode;
use crate::instructions::operands::VectorGPR;
use crate::{DisplayInstruction, DisplayableInstruction};
use bits::FromBits;
use bits_macros::FromBits;
use strum::AsRefStr;

#[derive(Debug, FromBits)]
#[bits(32)]
pub struct VINTRPInstruction {
    #[bits(17, 16)]
    op: VINTRPOpCode,

    #[bits(7, 0)]
    vsrc: VectorGPR,

    #[bits(9, 8)]
    attribute_channel: AttributeChannel,

    #[bits(15, 10)]
    attr: Attr,

    #[bits(25, 18)]
    vdst: VectorGPR,
}

#[derive(Debug, AsRefStr)]
enum AttributeChannel {
    #[strum(serialize = "x")]
    X,
    #[strum(serialize = "y")]
    Y,
    #[strum(serialize = "z")]
    Z,
    #[strum(serialize = "w")]
    W,
}

impl FromBits<2> for AttributeChannel {
    fn from_bits(value: usize) -> Self {
        match value {
            0 => AttributeChannel::X,
            1 => AttributeChannel::Y,
            2 => AttributeChannel::Z,
            3 => AttributeChannel::W,
            _ => unreachable!("unexpected value {}", value),
        }
    }
}

#[derive(Debug)]
struct Attr(u8);

impl FromBits<6> for Attr {
    fn from_bits(value: usize) -> Self {
        Self(value as _)
    }
}

impl DisplayInstruction for VINTRPInstruction {
    fn display(&self) -> DisplayableInstruction {
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
