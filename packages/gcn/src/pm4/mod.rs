mod op_codes;
mod registers;

use crate::bitrange::BitRange;
pub use crate::pm4::op_codes::OpCode;
pub use crate::pm4::registers::Register;
use crate::reader::Reader;
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
    SetContextRegister {
        values: Vec<ContextRegisterSetOperation>,
    },
    SetShaderRegister {
        values: Vec<ShaderRegisterSetOperation>,
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

impl EndOfPipePacket {
    pub fn from_type3_packet(body: Vec<u32>) -> EndOfPipePacket {
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

#[derive(Debug)]
pub struct ContextRegisterSetOperation {
    pub register: Option<Register>,
    pub value: u32,
}

#[derive(Debug)]
pub struct ShaderRegisterSetOperation {
    register: Option<Register>,
    value: u32,
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

                            ContextRegisterSetOperation {
                                register: Register::from_repr(((offset as usize) << 2) + 0x028000),
                                value,
                            }
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

                            ShaderRegisterSetOperation {
                                register: Register::from_repr(((offset as usize) << 2) + 0xB000),
                                value,
                            }
                        })
                        .collect(),
                }
            }
            OpCode::EVENT_WRITE_EOP => {
                Type3PacketValue::EndOfPipe(EndOfPipePacket::from_type3_packet(body))
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

/// Creates a bit range spanning from lowest to highest, inclusive. A value of
/// 0 indicates the least significant bit.
///
/// This is how bit ranges are specified in the AMD Southern Island PM4 docs.
pub fn bitrange(highest: u8, lowest: u8) -> BitRange {
    let bit_len = highest - lowest + 1;
    // todo: this 32 here is kinda wrong
    let start = 32 - bit_len - lowest;

    crate::bitrange::bitrange(start, bit_len)
}

#[cfg(test)]
mod tests {
    use crate::pm4::bitrange;

    #[test]
    fn test() {
        println!("{}", bitrange(15, 8).of_32(0xc0026900));
        assert_eq!(bitrange(31, 30), crate::bitrange::bitrange(0, 2));
        assert_eq!(bitrange(29, 16), crate::bitrange::bitrange(2, 14));
        assert_eq!(bitrange(15, 0), crate::bitrange::bitrange(16, 16));
        assert_eq!(bitrange(29, 0), crate::bitrange::bitrange(2, 30));

        assert_eq!(bitrange(15, 8), crate::bitrange::bitrange(16, 8));
        assert_eq!(bitrange(7, 2), crate::bitrange::bitrange(24, 6));
        assert_eq!(bitrange(1, 1), crate::bitrange::bitrange(30, 1));
        assert_eq!(bitrange(0, 0), crate::bitrange::bitrange(31, 1));
    }
}
