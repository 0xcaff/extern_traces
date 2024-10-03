use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::COMPUTE_DISPATCH_INITIATOR;
use alloc::vec::Vec;
use bits::TryFromBitsContainer;

#[derive(Debug, Clone)]
pub struct DispatchDirectPacket {
    pub dim_x: u32,
    pub dim_y: u32,
    pub dim_z: u32,

    pub initiator: COMPUTE_DISPATCH_INITIATOR,
}

impl ParseType3Packet for DispatchDirectPacket {
    const OP: OpCode = OpCode::DISPATCH_DIRECT;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            dim_x: body[0],
            dim_y: body[1],
            dim_z: body[2],
            initiator: COMPUTE_DISPATCH_INITIATOR::try_from_bits_container(body[3]).unwrap(),
        }
    }
}
