use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::COMPUTE_DISPATCH_INITIATOR;
use alloc::vec::Vec;
use bits::{Bits, TryFromBits, TryFromBitsContainer};
use strum::FromRepr;
use custom_debug::Debug;

#[derive(FromRepr, Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum BaseIndex {
    DisplayListPatchTableBase = 0b0000,
    DrawIndexIndirPatchTableBase = 0b0001,
    GDSPartitionBases = 0b0010,
    CEPartitionBases = 0b0011,
}

impl TryFromBits<4> for BaseIndex {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        BaseIndex::from_repr(value.full() as _)
    }
}

#[derive(Debug, Clone)]
pub struct DispatchIndirectPacket {
    #[debug(format = "{:#x}")]
    pub data_offset: u32,
    pub dispatch_initiator: COMPUTE_DISPATCH_INITIATOR,
}

impl ParseType3Packet for DispatchIndirectPacket {
    const OP: OpCode = OpCode::DISPATCH_INDIRECT;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let dispatch_initiator =
            COMPUTE_DISPATCH_INITIATOR::try_from_bits_container(body[1]).unwrap();

        let data_offset = body[0];

        Self {
            data_offset,
            dispatch_initiator,
        }
    }
}
