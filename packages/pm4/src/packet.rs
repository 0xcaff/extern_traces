use crate::op_codes::OpCode;
use crate::reader::Reader;
use crate::registers::ParseRegisterEntry;
use crate::{Register, RegisterEntry, VGT_DRAW_INITIATOR, VGT_EVENT_INITIATOR};
use bits::{bit, bitrange};
use bits::FromBits;
use std::io::Cursor;

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
    SetContextRegister { values: Vec<Option<RegisterEntry>> },
    SetShaderRegister { values: Vec<Option<RegisterEntry>> },
    EndOfPipe(EndOfPipePacket),
    WaitRegisterMemory(WaitRegisterMemoryPacket),
    DrawIndexAuto(DrawIndexAutoPacket),
    // todo: index_type
    // todo: set_uconfig_register
    Unknown { opcode: OpCode, body: Vec<u32> },
}

#[derive(Debug, Clone)]
pub struct EndOfPipePacket {
    pub invalidate_writeback_l2: bool,
    pub event_index: u8,
    pub event_type: VGT_EVENT_INITIATOR,
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
        let event_type = VGT_EVENT_INITIATOR::from_bits(bitrange(5, 0).of_32(event_control));

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

#[derive(Debug)]
pub enum Engine {
    /// CP.PFP
    PrefetchParser,

    /// CP.ME
    MicroEngine,
}

#[derive(Debug)]
pub enum MemoryAddress {
    Register(u32),
    Memory(u64),
}

#[derive(Debug)]
pub enum Function {
    Always,
    LessThan,
    LessThanEqual,
    Equal,
    NotEqual,
    GreaterThanEqual,
    GreaterThan,
}

#[derive(Debug)]
pub struct WaitRegisterMemoryPacket {
    pub engine: Engine,
    pub function: Function,

    pub tcl1_volatile_action_enable: bool,
    pub texture_cache_volatile_action_enable: bool,
    pub texture_cache_write_back_action_enable: bool,
    pub color_buffer_action_enable: bool,
    pub depth_buffer_action_enable: bool,

    pub reference_value: u32,
    pub poll_address_swap: u8,
    pub poll_address: MemoryAddress,
    pub mask: u32,
    pub poll_interval: u16,
}

impl ParseType3Packet for WaitRegisterMemoryPacket {
    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            engine: match bitrange(8, 8).of_32(body[0]) {
                0 => Engine::MicroEngine,
                1 => Engine::PrefetchParser,
                _ => unreachable!(),
            },
            poll_address: match bitrange(4, 4).of_32(body[0]) {
                0 => MemoryAddress::Register(bitrange(15, 0).of_32(body[1]) as _),
                1 => MemoryAddress::Memory(
                    (bitrange(31, 2).of_32(body[1]) | bitrange(15, 0).of_32(body[2]) << 30) as _,
                ),
                _ => unreachable!(),
            },
            poll_address_swap: bitrange(1, 0).of_32(body[1]) as _,
            function: match bitrange(2, 0).of_32(body[0]) {
                0b000 => Function::Always,
                0b001 => Function::LessThan,
                0b010 => Function::LessThanEqual,
                0b011 => Function::Equal,
                0b100 => Function::NotEqual,
                0b101 => Function::GreaterThanEqual,
                0b110 => Function::GreaterThan,
                value => panic!("unexpected value {}", value),
            },

            tcl1_volatile_action_enable: bit(15, body[0] as _),
            texture_cache_volatile_action_enable: bit(16, body[0] as _),
            texture_cache_write_back_action_enable: bit(18, body[0] as _),
            color_buffer_action_enable: bit(25, body[0] as _),
            depth_buffer_action_enable: bit(26, body[0] as _),

            reference_value: body[3],
            mask: body[4],
            poll_interval: bitrange(15, 0).of_32(body[5]) as _,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawIndexAutoPacket {
    index_count: u32,
    draw_initiator: VGT_DRAW_INITIATOR,
}

impl ParseType3Packet for DrawIndexAutoPacket {
    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            index_count: body[0],
            draw_initiator: VGT_DRAW_INITIATOR::from_bits(body[1] as _),
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
            OpCode::ACQUIRE_MEM => Type3PacketValue::WaitRegisterMemory(
                WaitRegisterMemoryPacket::parse_type3_packet(body),
            ),
            OpCode::DRAW_INDEX_AUTO => {
                Type3PacketValue::DrawIndexAuto(DrawIndexAutoPacket::parse_type3_packet(body))
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
