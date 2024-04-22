use bits::FromBits;
use crate::op_codes::OpCode;
use crate::{COMMAND, DMA_DATA_WORD0_cik, ParseType3Packet};

#[derive(Debug)]
pub struct DirectMemoryAccessPacket {
    fields: DMA_DATA_WORD0_cik,
    src_address: u64,
    dst_address: u64,
    command: COMMAND,
}

impl ParseType3Packet for DirectMemoryAccessPacket {
    const OP: OpCode = OpCode::DMA_DATA;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let fields = DMA_DATA_WORD0_cik::from_bits(body[0]);
        let src_address = (body[1] as u64) | ((body[2] as u64) << 32);
        let dst_address = (body[3] as u64) | ((body[4] as u64) << 32);
        let command = COMMAND::from_bits(body[5]);

        Self {
            fields,
            src_address,
            dst_address,
            command,
        }
    }
}