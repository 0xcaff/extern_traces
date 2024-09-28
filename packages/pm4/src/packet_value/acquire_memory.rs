use alloc::vec::Vec;
use bits::FromBits;

use crate::op_codes::OpCode;
use crate::{ParseType3Packet, CP_COHER_CNTL};

#[derive(Debug)]
pub struct AcquireMemoryPacket {
    pub command_processor_cache_coherence_control: CP_COHER_CNTL,
    pub command_processor_cache_coherence_size: u64,
    pub command_processor_cache_coherence_base: u64,
    pub poll_interval: u32,
}

impl ParseType3Packet for AcquireMemoryPacket {
    const OP: OpCode = OpCode::ACQUIRE_MEM;

    fn parse_type3_packet(body: Vec<u32>) -> Self {
        let command_processor_cache_coherence_control = CP_COHER_CNTL::from_bits(body[0]);
        let command_processor_cache_coherence_size = (body[1] as u64) | ((body[2] as u64) << 32);
        let command_processor_cache_coherence_base = (body[3] as u64) | ((body[4] as u64) << 32);
        let poll_interval = body[5];

        Self {
            command_processor_cache_coherence_control,
            command_processor_cache_coherence_size,
            command_processor_cache_coherence_base,
            poll_interval,
        }
    }
}
