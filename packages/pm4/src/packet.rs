use crate::op_codes::OpCode;
use crate::reader::Reader;
use bits::bitrange;
use std::io::Cursor;
use crate::{Register, RegisterEntry};
use crate::registers::ParseRegisterEntry;

#[derive(Debug)]
pub enum PM4Packet {
    Type0(Type0Packet),
    Type2(Type2Header),
    Type3(Type3Packet),
}

impl PM4Packet {
    pub fn parse(mut reader: impl Reader) -> Result<PM4Packet, anyhow::Error> {
        let value = reader.read_u32()?;
        let packet_type = bitrange(31, 30).of_32(value);
        let count = bitrange(29, 16).of_32(value) as u16 + 1;

        match packet_type {
            0 => Ok(PM4Packet::Type0(Type0Packet {
                base_idx: bitrange(15, 0).of_32(value) as u16,
                body: reader.read_dwords(count as usize)?,
            })),
            2 => Ok(PM4Packet::Type2(Type2Header {
                reserved: bitrange(29, 0).of_32(value) as u32,
            })),
            3 => Ok(PM4Packet::Type3(Type3Packet {
                header: Type3Header {
                    reserved: bitrange(7, 2).of_32(value) as u8,
                    shader_type: if bitrange(1, 1).of_32(value) == 0 {
                        ShaderType::Graphics
                    } else {
                        ShaderType::Compute
                    },
                    predicate: if bitrange(0, 0).of_32(value) == 0 {
                        false
                    } else {
                        true
                    },
                },
                value: Type3PacketValue::parse(
                    OpCode::from_op(bitrange(15, 8).of_32(value) as u8)?,
                    reader.read_dwords(count as usize)?,
                ),
            })),
            _ => panic!("unexpected packet type {}", packet_type),
        }
    }

    pub fn parse_all(draw_command_buffer: &[u8]) -> Result<Vec<PM4Packet>, anyhow::Error> {
        let mut cursor = Cursor::new(draw_command_buffer);

        let mut result = Vec::new();

        while !cursor.is_empty() {
            let packet = PM4Packet::parse(&mut cursor)?;

            result.push(packet);
        }

        Ok(result)
    }
}

#[derive(Debug)]
pub struct Type0Packet {
    pub base_idx: u16,
    pub body: Vec<u32>,
}

#[derive(Debug)]
pub struct Type2Header {
    pub reserved: u32,
}

#[derive(Debug)]
pub struct Type3Packet {
    pub header: Type3Header,
    pub value: Type3PacketValue,
}

#[derive(Debug)]
pub enum Type3PacketValue {
    SetContextRegister {
        values: Vec<Option<RegisterEntry>>,
    },
    SetShaderRegister {
        values: Vec<Option<RegisterEntry>>,
    },
    EndOfPipe(EndOfPipePacket),
    Unknown {
        opcode: OpCode,
        body: Vec<u32>,
    },
}

#[derive(Debug)]
pub struct EndOfPipePacket {
    pub invalidate_writeback_l2: bool,
    pub event_index: u8,
    pub event_type: u8,
    pub address: u64,
    pub immediate: u64,
}

trait ParseType3Packet {
    fn parse_type3_packet(body: Vec<u32>) -> Self;
}

impl ParseType3Packet for EndOfPipePacket {
    fn parse_type3_packet(body: Vec<u32>) -> EndOfPipePacket {
        let event_control = body[0];

        let inv_l2 = if bitrange(20, 20).of_32(event_control) == 1 {
            true
        } else {
            false
        };
        let event_index = bitrange(11, 8).of_32(event_control) as u8;
        let event_type = bitrange(5, 0).of_32(event_control) as u8;

        let address = (body[1] as u64) | ((bitrange(15, 0).of_32(body[2]) << 32) as u64);
        let immediate = (body[3] as u64) | ((body[4] as u64) << 32);

        EndOfPipePacket {
            invalidate_writeback_l2: inv_l2,
            event_index,
            event_type,

            address,
            immediate,
        }
    }
}

impl Type3PacketValue {
    pub fn parse(opcode: OpCode, mut body: Vec<u32>) -> Type3PacketValue {
        match opcode {
            OpCode::SET_CONTEXT_REG => {
                let value_header = body.remove(0);
                let offset = bitrange(15, 0).of_32(value_header) as u16;

                Type3PacketValue::SetContextRegister {
                    values: body
                        .into_iter()
                        .enumerate()
                        .map(|(idx, value)| {
                            let offset = offset + idx as u16;

                            let register = Register::from_repr(((offset as usize) << 2) + 0x028000);
                            register.map(|it| RegisterEntry::parse_register_entry(it, value))
                        })
                        .collect(),
                }
            }
            OpCode::SET_SH_REG => {
                let value_header = body.remove(0);
                let offset = bitrange(15, 0).of_32(value_header) as u16;

                Type3PacketValue::SetShaderRegister {
                    values: body
                        .into_iter()
                        .enumerate()
                        .map(|(idx, value)| {
                            let offset = offset + idx as u16;

                            let register = Register::from_repr(((offset as usize) << 2) + 0xB000);
                            register.map(|it| RegisterEntry::parse_register_entry(it, value))
                        })
                        .collect(),
                }
            }
            OpCode::EVENT_WRITE_EOP => {
                Type3PacketValue::EndOfPipe(EndOfPipePacket::parse_type3_packet(body))
            }
            _ => Type3PacketValue::Unknown { opcode, body },
        }
    }
}

#[derive(Debug)]
pub struct Type3Header {
    pub reserved: u8,
    pub shader_type: ShaderType,
    pub predicate: bool,
}

#[derive(Debug)]
pub enum ShaderType {
    Graphics,
    Compute,
}
