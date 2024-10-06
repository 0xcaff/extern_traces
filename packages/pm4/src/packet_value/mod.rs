use crate::dispatch_indirect::DispatchIndirectPacket;
use crate::draw_index_indirect::DrawIndexIndirectPacket;
use crate::event_write::EventWritePacket;
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
use crate::packet_value::set_base::SetBasePacket;
use crate::packet_value::wait_register_memory::WaitRegisterMemoryPacket;
use crate::register::SetUConfigRegisterPacket;
use alloc::vec::Vec;
use pm4_internal_macros::ParsePacketValue;
use crate::index_type::IndexTypePacket;

pub mod acquire_memory;
pub mod clear_state;
pub mod direct_memory_access;
pub mod dispatch_direct;
pub mod dispatch_indirect;
pub mod draw_index_2;
pub mod draw_index_auto;
pub mod draw_index_indirect;
pub mod event_write;
pub mod event_write_end_of_pipe;
pub mod event_write_end_of_shader;
pub mod indirect_buffer;
pub mod register;
pub mod release_memory;
pub mod set_base;
pub mod wait_register_memory;
pub mod index_type;

#[derive(Debug, ParsePacketValue)]
pub enum Type3PacketValue {
    SetContextRegister(SetContextRegisterPacket),
    SetShaderRegister(SetShaderRegisterPacket),
    SetUConfigRegister(SetUConfigRegisterPacket),
    EventWriteEndOfPipe(EventWriteEndOfPipePacket),
    DrawIndexAuto(DrawIndexAutoPacket),
    DrawIndexIndirect(DrawIndexIndirectPacket),
    DrawIndex2(DrawIndex2Packet),
    DispatchDirect(DispatchDirectPacket),
    DispatchIndirect(DispatchIndirectPacket),
    EventWriteEndOfShader(EventWriteEndOfShaderPacket),
    ClearState(ClearStatePacket),
    DirectMemoryAccess(DirectMemoryAccessPacket),
    AcquireMemory(AcquireMemoryPacket),
    IndirectBuffer(IndirectBufferPacket),
    ReleaseMemory(ReleaseMemoryPacket),
    WaitRegisterMemory(WaitRegisterMemoryPacket),
    SetBase(SetBasePacket),
    EventWrite(EventWritePacket),
    IndexType(IndexTypePacket),
    Unknown { op: OpCode, body: Vec<u32> },
}

pub trait ParsePacketValue {
    fn parse(op_code: OpCode, body: Vec<u32>) -> Self;
}

pub trait ParseType3Packet {
    const OP: OpCode;

    fn parse_type3_packet(body: Vec<u32>) -> Self;
}
