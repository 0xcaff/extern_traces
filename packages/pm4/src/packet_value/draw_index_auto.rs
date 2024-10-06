use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::VGT_DRAW_INITIATOR;
use alloc::vec::Vec;
use bits::TryFromBitsContainer;

#[derive(Debug, Clone)]
pub struct DrawIndexAutoPacket {
    pub index_count: u32,
    pub draw_initiator: VGT_DRAW_INITIATOR,
}

impl ParseType3Packet for DrawIndexAutoPacket {
    const OP: OpCode = OpCode::DRAW_INDEX_AUTO;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            index_count: body[0],
            draw_initiator: VGT_DRAW_INITIATOR::try_from_bits_container(body[1]).unwrap(),
        }
    }
}
