use crate::op_codes::OpCode;
use crate::reader::Reader;
use crate::registers::ParseRegisterEntry;
use crate::{Register, RegisterEntry, VGT_DRAW_INITIATOR, VGT_EVENT_INITIATOR};
use bits::FromBits;
use bits::{bit, bitrange};
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
    pub index_count: u32,
    pub draw_initiator: VGT_DRAW_INITIATOR,
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

#[cfg(test)]
mod tests {
    use crate::PM4Packet;
    use hex_literal::hex;

    #[test]
    fn test() {
        let bytes = hex!("002801c00000008000000080001200c000000000005805c0c07fc42effffffff0000000000000000000000000a000000007601c016020000ffffffff007601c017020000ffffffff007601c01502000000000000006901c0f90200002d000000006901c08202000008000000006901c08002000008000800006901c0810200000000ffff006901c00402000000000000006901c0060200003f040000006901c083000000ffff0000006901c01703000010000000006901c0fa0200000000803f006901c0fc0200000000803f006901c0fb0200000000803f006901c0fd0200000000803f006901c0020200001000cc00006901c00e030000ffffffff006901c00f030000ffffffff002f00c001000000007601c007000000ff010000007601c046000000ff010000007601c087000000ff010000007601c0c7000000ff010000007601c00701000000000000007601c047010000ff010000006901c0b101000002000000006901c00101000000000000006901c000010000ffffffff006901c00301000000000000006901c08402000000000000006901c09002000000000000006901c0ae02000000000000006901c09202000000000000006901c09302000000000006006901c0f802000000000000006901c0de020000e9010000006903c095020000000100000001000004000000007901c000020000000000e000107fc00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c05c013000000f03ae02af57f000000000000ffffffff0a000000006901c0d502000000000000006901c02000000041b24f80006901c0ab02000080000000006904c0d702000010000000000000000000000000000000006903c098020000000400000004000000040000006901c0ce02000040000000006901c0ac02000000040000006906c01803000000604c80ef00f00e7f7f000000000000288802004a010000006904c01f03000000000000000000000000000000000000001000c080073804006901c02b03000000000000006901c03a03000000000000006901c04903000000000000006901c05803000000000000006901c06703000000000000006901c07603000000000000006901c08503000000000000006901c01000000000000000006901c01100000000000000006901c0a502000001000000006901c003010000ffff0000006901c00b01000000000000006901c00402000000000000006901c00000000000000000006901c00b0000000000803f006901c00a00000000000000002a00c000000000006901c00201000000000000006901c0e001000004050161006901c0e101000004050121006901c0e201000004050121006901c0e301000004050121006901c0e401000004050121006901c0e501000004050121006901c0e601000004050121006901c0e701000004050121006904c0050100000000803f0000803f0000803f0000803f007901c04202000000000000006901c00002000030000000006902c00c01000000ffff0100ffff01006901c08e0000000f000000006902c0b4000000000000000000803f006906c00f0100000000704400007044000007c4000007440000003f0000003f006901c0060200003f040000006902c00c0000000000000080073804006901c08d0000003c002200006901c0fa02000040b26e42006901c0fc02000087880442006901c0fb0200000000803f006901c0fd0200000000803f006902c00c0000000000000080073804006901c00502000040020000004704c004050000004210c5800000600000000000000000004704c004050000084210c5800000600000000000000000004704c004050000203d10c5800000200000000000000000004704c0040500005035873e80000042010000000000000000103ec0770775680000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

        insta::assert_debug_snapshot!(PM4Packet::parse_all(&bytes));
    }
}
