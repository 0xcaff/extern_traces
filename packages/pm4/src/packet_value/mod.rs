use crate::indirect_buffer::IndirectBufferPacket;
use crate::op_codes::OpCode;
use crate::packet_value::acquire_memory::AcquireMemoryPacket;
use crate::packet_value::clear_state::ClearStatePacket;
use crate::packet_value::direct_memory_access::DirectMemoryAccessPacket;
use crate::packet_value::dispatch_direct::DispatchDirectPacket;
use crate::packet_value::draw_index_2::DrawIndex2Packet;
use crate::packet_value::draw_index_auto::DrawIndexAutoPacket;
use crate::packet_value::event_write_end_of_pipe::EventWriteEndOfPipePacket;
use crate::packet_value::event_write_end_of_shader::EventWriteEndOfShaderPacket;
use crate::packet_value::register::{SetContextRegisterPacket, SetShaderRegisterPacket};
use crate::packet_value::release_memory::ReleaseMemoryPacket;
use crate::packet_value::wait_register_memory::WaitRegisterMemoryPacket;
use crate::register::SetUConfigRegisterPacket;
use alloc::vec::Vec;
use pm4_internal_macros::ParsePacketValue;

pub mod acquire_memory;
pub mod clear_state;
pub mod direct_memory_access;
pub mod dispatch_direct;
pub mod draw_index_2;
pub mod draw_index_auto;
pub mod event_write_end_of_pipe;
pub mod event_write_end_of_shader;
pub mod indirect_buffer;
pub mod register;
pub mod release_memory;
pub mod wait_register_memory;

#[derive(Debug, ParsePacketValue)]
pub enum Type3PacketValue {
    SetContextRegister(SetContextRegisterPacket),
    SetShaderRegister(SetShaderRegisterPacket),
    SetUConfigRegister(SetUConfigRegisterPacket),
    EventWriteEndOfPipe(EventWriteEndOfPipePacket),
    DrawIndexAuto(DrawIndexAutoPacket),
    DrawIndex2(DrawIndex2Packet),
    DispatchDirect(DispatchDirectPacket),
    EventWriteEndOfShader(EventWriteEndOfShaderPacket),
    ClearState(ClearStatePacket),
    DirectMemoryAccess(DirectMemoryAccessPacket),
    AcquireMemory(AcquireMemoryPacket),
    IndirectBuffer(IndirectBufferPacket),
    ReleaseMemory(ReleaseMemoryPacket),
    WaitRegisterMemory(WaitRegisterMemoryPacket),
    // todo: index_type
    Unknown { op: OpCode, body: Vec<u32> },
}

pub trait ParsePacketValue {
    fn parse(op_code: OpCode, body: Vec<u32>) -> Self;
}

pub trait ParseType3Packet {
    const OP: OpCode;

    fn parse_type3_packet(body: Vec<u32>) -> Self;
}
