use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::VGT_DRAW_INITIATOR;
use alloc::vec::Vec;
use bits::TryFromBitsContainer;

#[derive(Debug, Clone)]
pub struct DrawIndex2Packet {
    pub max_size: u32,
    pub index_base: u64,
    pub index_count: u32,
    pub draw_initiator: VGT_DRAW_INITIATOR,
}

impl ParseType3Packet for DrawIndex2Packet {
    const OP: OpCode = OpCode::DRAW_INDEX_2;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            max_size: body[0],
            index_base: (body[1] as u64) | ((body[2] as u64) << 32),
            index_count: body[3],
            draw_initiator: VGT_DRAW_INITIATOR::try_from_bits_container(body[4]).unwrap(),
        }
    }
}
