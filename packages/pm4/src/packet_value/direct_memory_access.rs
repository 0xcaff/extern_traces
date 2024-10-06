use crate::op_codes::OpCode;
use crate::{DMA_DATA_WORD0_cik, ParseType3Packet, COMMAND};
use alloc::vec::Vec;
use bits::TryFromBitsContainer;

#[derive(Debug)]
pub struct DirectMemoryAccessPacket {
    pub fields: DMA_DATA_WORD0_cik,
    pub src_address: u64,
    pub dst_address: u64,
    pub command: COMMAND,
}

impl ParseType3Packet for DirectMemoryAccessPacket {
    const OP: OpCode = OpCode::DMA_DATA;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let fields = DMA_DATA_WORD0_cik::try_from_bits_container(body[0]).unwrap();
        let src_address = (body[1] as u64) | ((body[2] as u64) << 32);
        let dst_address = (body[3] as u64) | ((body[4] as u64) << 32);
        let command = COMMAND::try_from_bits_container(body[5]).unwrap();

        Self {
            fields,
            src_address,
            dst_address,
            command,
        }
    }
}
