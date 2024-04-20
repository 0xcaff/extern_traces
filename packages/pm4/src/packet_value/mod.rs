use crate::op_codes::OpCode;
use crate::packet_value::dispatch_direct::DispatchDirectPacket;
use crate::packet_value::draw_index_auto::DrawIndexAutoPacket;
use crate::packet_value::event_write_end_of_pipe::EventWriteEndOfPipePacket;
use crate::packet_value::event_write_end_of_shader::EventWriteEndOfShaderPacket;
use crate::packet_value::register::{SetContextRegisterPacket, SetShaderRegisterPacket};
use crate::register::SetUConfigRegisterPacket;
use pm4_internal_macros::ParsePacketValue;

pub mod dispatch_direct;
pub mod draw_index_auto;
pub mod event_write_end_of_pipe;
pub mod event_write_end_of_shader;
pub mod register;

#[derive(Debug, ParsePacketValue)]
pub enum Type3PacketValue {
    SetContextRegister(SetContextRegisterPacket),
    SetShaderRegister(SetShaderRegisterPacket),
    SetUConfigRegister(SetUConfigRegisterPacket),
    EventWriteEndOfPipe(EventWriteEndOfPipePacket),
    DrawIndexAuto(DrawIndexAutoPacket),
    DispatchDirect(DispatchDirectPacket),
    EventWriteEndOfShader(EventWriteEndOfShaderPacket),
    // todo: index_type
    // todo: acquire memory
    Unknown { op: OpCode, body: Vec<u32> },
}

pub trait ParsePacketValue {
    fn parse(op_code: OpCode, body: Vec<u32>) -> Self;
}

pub trait ParseType3Packet {
    const OP: OpCode;

    fn parse_type3_packet(body: Vec<u32>) -> Self;
}
