use crate::packet_value::ParseType3Packet;
use crate::VGT_DRAW_INITIATOR;
use bits::FromBits;

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
