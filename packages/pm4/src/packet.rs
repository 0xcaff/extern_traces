use crate::op_codes::OpCode;
use crate::packet_value::{ParsePacketValue, Type3PacketValue};
use crate::reader::Reader;
use bits::bitrange;
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
    use crate::{convert, PM4Packet};

    #[test]
    fn unknown() {
        let bytes = &include_bytes!("./test_data/unknown.pm4")[..];
        let packets = PM4Packet::parse_all(bytes).unwrap();

        insta::assert_debug_snapshot!(packets);
        insta::assert_debug_snapshot!(convert(packets.as_slice()));
    }

    #[test]
    fn compute_and_graphics() {
        let bytes = &include_bytes!("./test_data/compute_and_graphics.pm4")[..];
        let packets = PM4Packet::parse_all(bytes).unwrap();

        insta::assert_debug_snapshot!(packets);
        insta::assert_debug_snapshot!(convert(packets.as_slice()));
    }
}
