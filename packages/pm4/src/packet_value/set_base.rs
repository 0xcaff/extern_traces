use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use alloc::vec::Vec;
use bits::{bitrange, Bits, TryFromBits};
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
pub struct SetBasePacket {
    #[debug(format = "0x{:x}")]
    pub address: u64,
}

impl ParseType3Packet for SetBasePacket {
    const OP: OpCode = OpCode::SET_BASE;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let base_index = BaseIndex::try_from_bits(bitrange(3, 0).of(body[0] as _)).unwrap();
        assert_eq!(BaseIndex::DrawIndexIndirPatchTableBase, base_index);

        let address = body[1] as u64 | ((bitrange(15, 0).of(body[2] as _)) << 32);

        Self {
            address,
        }
    }
}
