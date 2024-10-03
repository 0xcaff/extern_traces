use alloc::vec::Vec;
use bits::{bit, bitrange, Bits, TryFromBitsContainer};
use custom_debug::Debug;
use strum::FromRepr;

use crate::op_codes::OpCode;
use crate::{ParseType3Packet, VGT_EVENT_INITIATOR};

#[derive(Debug)]
pub struct EventWritePacket {
    pub inv_l2: bool,
    pub event_index: EventIndex,
    pub event_type: VGT_EVENT_INITIATOR,
    pub address: Option<u64>,
}

#[derive(FromRepr, Debug)]
#[repr(u8)]
pub enum EventIndex {
    ZPassDone = 0b001,
    SamplePipelinesAt = 0b0010,
    SampleStreamOutsAt = 0b0011,
    PartialFlush = 0b100,
    CacheFlush = 0b111,
}

impl EventIndex {
    pub fn needs_special_handling(&self) -> bool {
        match self {
            EventIndex::ZPassDone
            | EventIndex::SamplePipelinesAt
            | EventIndex::SampleStreamOutsAt => true,
            EventIndex::PartialFlush | EventIndex::CacheFlush => false,
        }
    }
}

impl ParseType3Packet for EventWritePacket {
    const OP: OpCode = OpCode::EVENT_WRITE;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let inv_l2 = bit(20, body[0] as _);
        let event_index = EventIndex::from_repr(bitrange(11, 8).of_32(body[0]) as _).unwrap();
        let event_type = VGT_EVENT_INITIATOR::try_from_bits_container(body[0].slice(5, 0)).unwrap();

        let address = if event_index.needs_special_handling() {
            let address_lo = body[1];
            let address_hi = body[2];

            let address = ((address_lo as u64) << 32) | (bitrange(15, 0).of_32(address_hi) as u64);

            Some(address)
        } else {
            None
        };

        Self {
            inv_l2,
            event_index,
            event_type,
            address,
        }
    }
}
