use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::registers::ParseRegisterEntry;
use crate::{Register, RegisterEntry};
use bits::bitrange;

#[derive(Debug)]
pub struct SetContextRegisterPacket {
    pub values: Vec<Option<RegisterEntry>>,
}

impl ParseType3Packet for SetContextRegisterPacket {
    const OP: OpCode = OpCode::SET_CONTEXT_REG;

    fn parse_type3_packet(mut body: Vec<u32>) -> Self {
        let value_header = body.remove(0);
        let offset = bitrange(15, 0).of_32(value_header) as u16;

        Self {
            values: body
                .into_iter()
                .enumerate()
                .map(|(idx, value)| {
                    let offset = offset + idx as u16;

                    let register = Register::from_repr(((offset as u64) << 2) + 0x028000);
                    register.map(|it| RegisterEntry::parse_register_entry(it, value))
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct SetShaderRegisterPacket {
    pub values: Vec<Option<RegisterEntry>>,
}

impl ParseType3Packet for SetShaderRegisterPacket {
    const OP: OpCode = OpCode::SET_SH_REG;

    fn parse_type3_packet(mut body: Vec<u32>) -> Self {
        let value_header = body.remove(0);
        let offset = bitrange(15, 0).of_32(value_header) as u16;

        Self {
            values: body
                .into_iter()
                .enumerate()
                .map(|(idx, value)| {
                    let offset = offset + idx as u16;

                    let register = Register::from_repr(((offset as u64) << 2) + 0xB000);
                    register.map(|it| RegisterEntry::parse_register_entry(it, value))
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct SetUConfigRegisterPacket {
    pub values: Vec<Option<RegisterEntry>>,
}

impl ParseType3Packet for SetUConfigRegisterPacket {
    const OP: OpCode = OpCode::SET_UCONFIG_REG;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        // Implemented looking at the writer side in mesa
        // https://gitlab.freedesktop.org/mesa/mesa/blob/d09ad16fd4a0596fb6c97cffaf0fdf031053b5a4/src/amd/vulkan/radv_cs.h#L160-L176
        Self {
            values: body
                .chunks(2)
                .map(|it| {
                    let &[key, value] = it else { unreachable!() };

                    // todo: not exactly sure what this field is, it always seems to be zero?
                    let idx = bitrange(31, 28).of_32(key);
                    assert_eq!(idx, 0);

                    let register = bitrange(27, 0).of_32(key);

                    let register = Register::from_repr(((register as u64) << 2) + 0x00030000);

                    register.map(|it| RegisterEntry::parse_register_entry(it, value))
                })
                .collect(),
        }
    }
}
