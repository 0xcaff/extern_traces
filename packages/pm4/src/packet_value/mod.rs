use crate::op_codes::OpCode;
pub use crate::packet_value::dispatch_direct::DispatchDirectPacket;
pub use crate::packet_value::draw_index_auto::DrawIndexAutoPacket;
pub use crate::packet_value::end_of_pipe::EndOfPipePacket;
pub use crate::packet_value::register::{SetContextRegisterPacket, SetShaderRegisterPacket};

mod dispatch_direct;
mod draw_index_auto;
mod end_of_pipe;
mod register;

#[derive(Debug)]
pub enum Type3PacketValue {
    SetContextRegister(SetContextRegisterPacket),
    SetShaderRegister(SetShaderRegisterPacket),
    EndOfPipe(EndOfPipePacket),
    DrawIndexAuto(DrawIndexAutoPacket),
    DispatchDirect(DispatchDirectPacket),
    // todo: index_type
    // todo: set_uconfig_register
    Unknown { opcode: OpCode, body: Vec<u32> },
}

impl Type3PacketValue {
    pub fn parse(opcode: OpCode, body: Vec<u32>) -> Type3PacketValue {
        match opcode {
            OpCode::SET_CONTEXT_REG => Type3PacketValue::SetContextRegister(
                SetContextRegisterPacket::parse_type3_packet(body),
            ),
            OpCode::SET_SH_REG => Type3PacketValue::SetShaderRegister(
                SetShaderRegisterPacket::parse_type3_packet(body),
            ),
            OpCode::EVENT_WRITE_EOP => {
                Type3PacketValue::EndOfPipe(EndOfPipePacket::parse_type3_packet(body))
            }
            OpCode::DRAW_INDEX_AUTO => {
                Type3PacketValue::DrawIndexAuto(DrawIndexAutoPacket::parse_type3_packet(body))
            }
            OpCode::DISPATCH_DIRECT => {
                Type3PacketValue::DispatchDirect(DispatchDirectPacket::parse_type3_packet(body))
            }
            _ => Type3PacketValue::Unknown { opcode, body },
        }
    }
}

pub trait ParseType3Packet {
    fn parse_type3_packet(body: Vec<u32>) -> Self;
}
