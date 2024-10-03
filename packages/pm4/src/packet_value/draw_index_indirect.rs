use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use alloc::vec::Vec;
use bits::bitrange;
use custom_debug::Debug;

#[derive(Debug, Clone)]
pub struct DrawIndexIndirectPacket {
    #[debug(format = "{:#x}")]
    pub data_offset: u32,
    pub base_vertex_location: u16,
}

impl ParseType3Packet for DrawIndexIndirectPacket {
    const OP: OpCode = OpCode::DRAW_INDEX_INDIRECT;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let data_offset = body[0];
        let base_vertex_location = bitrange(15, 0).of_32(body[1]);

        Self {
            data_offset,
            base_vertex_location: base_vertex_location as _,
            // todo: deal with this
            // draw_initiator: VGT_DRAW_INITIATOR::try_from_bits_container(body[2]).unwrap(),
        }
    }
}
