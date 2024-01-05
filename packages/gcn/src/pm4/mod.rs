use crate::bitrange::bitrange;
use crate::reader::{ReadError, Reader};
use anyhow::format_err;

#[derive(Debug)]
pub enum PM4Packet {
    Type0(Type0Packet),
    Type2(Type2Header),
    Type3(Type3Packet),
}

impl PM4Packet {
    pub fn parse(mut reader: impl Reader) -> Result<PM4Packet, ReadError> {
        let value = reader.read_u32()?;
        let packet_type = bitrange(0, 2).of_32(value);

        match packet_type {
            0 => {
                let count = bitrange(2, 14).of_32(value) as u16;

                let header = Type0Header {
                    count,
                    base_idx: bitrange(16, 16).of_32(value) as u16,
                };

                Ok(PM4Packet::Type0(Type0Packet {
                    header,
                    body: reader.read_bytes((count * 4) as usize)?,
                }))
            }
            1 => Err(format_err!("no").into()),
            2 => Ok(PM4Packet::Type2(Type2Header {
                reserved: bitrange(2, 30).of_32(value) as u32,
            })),
            3 => {
                let count = bitrange(2, 14).of_32(value) as u16;

                let header = Type3Header {
                    count,
                    opcode: bitrange(16, 8).of_32(value) as u8,
                    reserved: bitrange(24, 6).of_32(value) as u8,
                    shader_type: if bitrange(30, 1).of_32(value) == 0 {
                        ShaderType::Graphics
                    } else {
                        ShaderType::Compute
                    },
                    predicate: if bitrange(31, 1).of_32(value) == 0 {
                        false
                    } else {
                        true
                    },
                };

                Ok(PM4Packet::Type3(Type3Packet {
                    header,
                    body: reader.read_bytes((count * 4) as usize)?,
                }))
            }
            _ => panic!("unexpected packet type {}", packet_type),
        }
    }
}

#[derive(Debug)]
pub struct Type0Packet {
    pub header: Type0Header,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Type0Header {
    pub count: u16,
    pub base_idx: u16,
}

#[derive(Debug)]
pub struct Type2Header {
    pub reserved: u32,
}

#[derive(Debug)]
pub struct Type3Packet {
    pub header: Type3Header,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Type3Header {
    pub count: u16,
    pub opcode: u8,
    pub reserved: u8,
    pub shader_type: ShaderType,
    pub predicate: bool,
}

#[derive(Debug)]
pub enum ShaderType {
    Graphics,
    Compute,
}
