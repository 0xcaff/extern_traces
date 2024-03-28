use crate::op_codes::OpCode;
pub use crate::packet_value::dispatch_direct::DispatchDirectPacket;
pub use crate::packet_value::draw_index_auto::DrawIndexAutoPacket;
pub use crate::packet_value::end_of_pipe::EventWriteEndOfPipePacket;
pub use crate::packet_value::register::{SetContextRegisterPacket, SetShaderRegisterPacket};
use pm4_internal_macros::ParsePacketValue;

mod dispatch_direct;
mod draw_index_auto;
mod end_of_pipe;
mod register;

#[derive(Debug, ParsePacketValue)]
pub enum Type3PacketValue {
    SetContextRegister(SetContextRegisterPacket),
    SetShaderRegister(SetShaderRegisterPacket),
    EndOfPipe(EventWriteEndOfPipePacket),
    DrawIndexAuto(DrawIndexAutoPacket),
    DispatchDirect(DispatchDirectPacket),
    // todo: index_type
    // todo: set_uconfig_register
    Unknown { op: OpCode, body: Vec<u32> },
}

pub trait ParsePacketValue {
    fn parse(op_code: OpCode, body: Vec<u32>) -> Self;
}

pub trait ParseType3Packet {
    const OP: OpCode;

    fn parse_type3_packet(body: Vec<u32>) -> Self;
}
