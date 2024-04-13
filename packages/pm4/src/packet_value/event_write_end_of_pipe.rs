use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::VGT_EVENT_TYPE;
use bits::{bitrange, Bits, FromBits};

#[derive(Debug, Clone)]
pub struct EventWriteEndOfPipePacket {
    pub invalidate_writeback_l2: bool,
    pub event_index: u8,
    pub event_type: VGT_EVENT_TYPE,
    pub address: u64,
    pub immediate: u64,
}

impl ParseType3Packet for EventWriteEndOfPipePacket {
    const OP: OpCode = OpCode::EVENT_WRITE_EOP;

    fn parse_type3_packet(body: Vec<u32>) -> EventWriteEndOfPipePacket {
        let event_control = body[0];

        let inv_l2 = if bitrange(20, 20).of_32(event_control) == 1 {
            true
        } else {
            false
        };
        let event_index = bitrange(11, 8).of_32(event_control) as u8;
        let event_type = VGT_EVENT_TYPE::from_bits(event_control.slice(5, 0));

        let address = (body[1] as u64) | ((bitrange(15, 0).of_32(body[2]) << 32) as u64);
        let immediate = (body[3] as u64) | ((body[4] as u64) << 32);

        EventWriteEndOfPipePacket {
            invalidate_writeback_l2: inv_l2,
            event_index,
            event_type,

            address,
            immediate,
        }
    }
}
