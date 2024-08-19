use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use bits::bitrange;
use custom_debug::Debug;

#[derive(Debug, Clone)]
pub struct IndirectBufferPacket {
    #[debug(format = "0x{:x}")]
    pub virtual_address: u64,
    pub vmid: u32,
    pub command_buffer_size_dwords: u32,
}

impl ParseType3Packet for IndirectBufferPacket {
    const OP: OpCode = OpCode::INDIRECT_BUFFER_CIK;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            virtual_address: (body[0] as u64) | ((bitrange(15, 0).of_32(body[1]) as u64) << 32),
            vmid: bitrange(31, 24).of_32(body[2]) as _,
            command_buffer_size_dwords: bitrange(19, 0).of_32(body[2]) as _,
        }
    }
}
