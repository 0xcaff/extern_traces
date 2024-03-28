use crate::packet_value::ParseType3Packet;
use crate::VGT_EVENT_INITIATOR;
use bits::{bitrange, FromBits};

#[derive(Debug, Clone)]
pub struct EndOfPipePacket {
    pub invalidate_writeback_l2: bool,
    pub event_index: u8,
    pub event_type: VGT_EVENT_INITIATOR,
    pub address: u64,
    pub immediate: u64,
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
