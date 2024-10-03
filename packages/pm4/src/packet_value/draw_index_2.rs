use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::VGT_DRAW_INITIATOR;
use alloc::vec::Vec;
use bits::TryFromBitsContainer;
use custom_debug::Debug;

#[derive(Debug, Clone)]
pub struct DrawIndex2Packet {
    pub max_size: u32,
    #[debug(format = "{:#x}")]
    pub index_base: u64,
    pub index_count: u32,
    pub draw_initiator: VGT_DRAW_INITIATOR,
}

impl ParseType3Packet for DrawIndex2Packet {
    const OP: OpCode = OpCode::DRAW_INDEX_2;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let max_size = body[0];
        let index_count = body[3];

        assert_eq!(max_size, index_count);

        Self {
            max_size,
            index_base: (body[1] as u64) | ((body[2] as u64) << 32),
            index_count,
            draw_initiator: VGT_DRAW_INITIATOR::try_from_bits_container(body[4]).unwrap(),
        }
    }
}
