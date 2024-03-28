use crate::op_codes::OpCode;
use crate::{ParseType3Packet, VGT_EVENT_TYPE};
use bits::{bitrange, FromBits};

#[derive(Debug, Clone)]
pub struct EventWriteEndOfShaderPacket {
    pub event_type: VGT_EVENT_TYPE,
    pub address: u64,
    pub data: Data,
}

#[derive(Debug, Clone)]
pub enum Data {
    StorePacketData { data: u32 },
    LoadGDS { size: u16, gds_index: u16 },
}

impl ParseType3Packet for EventWriteEndOfShaderPacket {
    const OP: OpCode = OpCode::EVENT_WRITE_EOS;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let event_cntl = body[0];
        let event_index = bitrange(11, 8).of_32(event_cntl) as u8;
        assert_eq!(event_index, 0b0110);
        let event_type = bitrange(5, 0).of_32(event_cntl) as u8;

        let address_lo = bitrange(31, 2).of_32(body[1]);

        let cmd_info = body[2];
        let cmd = bitrange(31, 29).of_32(cmd_info);
        let address_hi = bitrange(15, 0).of_32(cmd_info);

        let data_info = body[3];

        Self {
            event_type: VGT_EVENT_TYPE::from_bits(event_type as usize),
            address: { (address_lo << 1) | (address_hi << 32) } as _,
            data: {
                match cmd {
                    0b010 => Data::StorePacketData { data: data_info },
                    0b001 => {
                        let size = bitrange(31, 16).of_32(data_info) as u16;
                        let gds_index = bitrange(15, 0).of_32(data_info) as u16;

                        Data::LoadGDS { size, gds_index }
                    }
                    _ => unimplemented!("unimplemented cmd value of 0b{:b}", cmd),
                }
            },
        }
    }
}
