use crate::op_codes::OpCode;
use crate::ParseType3Packet;

#[derive(Debug)]
pub struct ClearStatePacket;

impl ParseType3Packet for ClearStatePacket {
    const OP: OpCode = OpCode::CLEAR_STATE;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        assert_eq!(body.len(), 1);
        assert_eq!(body[0], 0);

        ClearStatePacket
    }
}
