use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use alloc::vec::Vec;
use bits::{Bits, FromBits, TryFromBitsContainer};
use bits_macros::TryFromBitsContainer;
use custom_debug::Debug;
use strum::FromRepr;

#[derive(Debug, Clone)]
pub struct WaitRegisterMemoryPacket {
    pub fields: Fields,

    #[debug(format = "0x{:x}")]
    pub poll_address_lo: u32,

    #[debug(format = "0x{:x}")]
    pub poll_address_hi: u32,
    pub reference: u32,

    #[debug(format = "0x{:x}")]
    pub mask: u32,
    pub poll_interval: u32,
}

#[derive(Debug, Clone)]
pub enum Engine {
    Memory,
    PrefetchParser,
}

impl FromBits<1> for Engine {
    fn from_bits(value: impl Bits) -> Self {
        match value.full() {
            0 => Engine::Memory,
            1 => Engine::PrefetchParser,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, TryFromBitsContainer, Clone)]
#[bits(32)]
pub struct Fields {
    #[bits(8, 8)]
    pub engine: Engine,

    #[bits(4, 4)]
    pub memory_space: MemorySpace,

    #[bits(2, 0)]
    pub function: Function,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MemorySpace {
    Register,
    Memory,
}

impl FromBits<1> for MemorySpace {
    fn from_bits(value: impl Bits) -> Self {
        match value.full() {
            0 => MemorySpace::Register,
            1 => MemorySpace::Memory,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, FromRepr, Clone)]
#[repr(usize)]
pub enum Function {
    Always = 0b000,
    LessThan = 0b001,
    LessThanEqual = 0b010,
    Equal = 0b011,
    NotEqual = 0b100,
    GreaterThanEqual = 0b101,
    GreaterThan = 0b110,
    Reserved = 0b111,
}

impl Function {
    pub fn compare(&self, value: u32, reference: u32) -> bool {
        match self {
            Function::Always => true,
            Function::LessThan => value < reference,
            Function::LessThanEqual => value <= reference,
            Function::Equal => value == reference,
            Function::NotEqual => value != reference,
            Function::GreaterThanEqual => value >= reference,
            Function::GreaterThan => value > reference,
            Function::Reserved => unimplemented!(),
        }
    }
}

impl FromBits<3> for Function {
    fn from_bits(value: impl Bits) -> Self {
        Function::from_repr(value.full() as _).unwrap()
    }
}

impl ParseType3Packet for WaitRegisterMemoryPacket {
    const OP: OpCode = OpCode::WAIT_REG_MEM;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            fields: Fields::try_from_bits_container(body[0]).unwrap(),
            poll_address_lo: body[1],
            poll_address_hi: body[2],
            reference: body[3],
            mask: body[4],
            poll_interval: body[5],
        }
    }
}
