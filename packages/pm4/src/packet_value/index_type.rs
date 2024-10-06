use crate::op_codes::OpCode;
use crate::ParseType3Packet;
use alloc::vec::Vec;
use bits::{bit, bitrange};

#[derive(Debug)]
pub struct IndexTypePacket {
    pub index_type: IndexType,
    pub swap_mode: u8,
}

#[derive(Debug)]
pub enum IndexType {
    U16,
    U32,
}

impl ParseType3Packet for IndexTypePacket {
    const OP: OpCode = OpCode::INDEX_TYPE;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        assert_eq!(body.len(), 1);

        IndexTypePacket {
            index_type: match bit(0, body[0] as _) {
                false => IndexType::U16,
                true => IndexType::U32,
            },
            swap_mode: bitrange(3, 2).of_32(body[0] as _) as _,
        }
    }
}
