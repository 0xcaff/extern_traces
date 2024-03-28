use crate::packet_value::ParseType3Packet;

#[derive(Debug)]
pub struct DispatchDirectPacket {
    pub dim_x: u32,
    pub dim_y: u32,
    pub dim_z: u32,

    // todo: parse initiator field
    pub initiator: u32,
}

impl ParseType3Packet for DispatchDirectPacket {
    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            dim_x: body[0],
            dim_y: body[1],
            dim_z: body[2],
            initiator: body[3],
        }
    }
}
