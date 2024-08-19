use crate::op_codes::OpCode;
use crate::packet_value::ParseType3Packet;
use crate::RELEASE_MEM_OP;
use bits::{Bits, FromBits};
use bits_macros::FromBits;
use strum::FromRepr;
use custom_debug::Debug;

#[derive(Debug)]
pub struct ReleaseMemoryPacket {
    pub op: RELEASE_MEM_OP,
    pub selector: Selectors,
    #[debug(format = "0x{:x}")]
    pub virtual_address: u64,
    pub immediate_data: u64,
}

// From Mesa
// https://gitlab.freedesktop.org/mesa/mesa/blob/d09ad16fd4a0596fb6c97cffaf0fdf031053b5a4/src/amd/common/sid.h#L178-L189
#[derive(Debug, Clone, FromBits)]
#[bits(32)]
pub struct Selectors {
    #[bits(17, 16)]
    pub destination_selection: DestinationSelection,

    #[bits(25, 24)]
    pub interrupt_selection: InterruptSelection,

    #[bits(31, 29)]
    pub data_selection: DataSelection,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, FromRepr)]
pub enum DestinationSelection {
    MEM = 0,
    TC_L2 = 1,
}

impl FromBits<2> for DestinationSelection {
    fn from_bits(value: impl Bits) -> Self {
        Self::from_repr(value.full() as _).unwrap()
    }
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, FromRepr)]
pub enum InterruptSelection {
    NONE = 0,
    UNKNOWN_1 = 2,
    SEND_DATA_AFTER_WR_CONFIRM = 3,
}

impl FromBits<2> for InterruptSelection {
    fn from_bits(value: impl Bits) -> Self {
        Self::from_repr(value.full() as _).unwrap()
    }
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, FromRepr)]
pub enum DataSelection {
    DISCARD = 0,
    VALUE_32BIT = 1,
    VALUE_64BIT = 2,
    TIMESTAMP = 3,
    UNKNOWN_1 = 4,
    GDS = 5,
}

impl FromBits<3> for DataSelection {
    fn from_bits(value: impl Bits) -> Self {
        Self::from_repr(value.full() as _).unwrap()
    }
}

impl ParseType3Packet for ReleaseMemoryPacket {
    const OP: OpCode = OpCode::RELEASE_MEM;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        Self {
            op: RELEASE_MEM_OP::from_bits(body[0]),
            selector: Selectors::from_bits(body[1]),
            virtual_address: (body[2] as u64) | ((body[3] as u64) << 32),
            immediate_data: (body[4] as u64) | ((body[5] as u64) << 32),
        }
    }
}
