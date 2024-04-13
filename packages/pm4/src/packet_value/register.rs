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
