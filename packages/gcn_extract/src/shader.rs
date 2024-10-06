use alloc::vec;
use alloc::vec::Vec;
use anyhow::bail;
use core::slice;
use gcn::instructions::formats::{FormattedInstruction, SOPPInstruction};
use gcn::instructions::ops::SOPPOpCode;
use gcn::instructions::Instruction;
use gcn::SliceReader;

pub struct ShaderInvocation {
    pub bytes: &'static [u32],
}

impl ShaderInvocation {
    pub unsafe fn decode_from_memory(entry_point: u32) -> Result<ShaderInvocation, anyhow::Error> {
        let entry_point = (entry_point as u64) << 8;

        let raw_code_pointer = entry_point as *const u32;
        let code_length = (*raw_code_pointer.offset(1) + 1) * 2;

        if slice::from_raw_parts(raw_code_pointer.offset(code_length as _) as *const u8, 7)
            != b"OrbShdr"
        {
            bail!("missing tag");
        }

        Ok(ShaderInvocation {
            bytes: slice::from_raw_parts(raw_code_pointer, code_length as usize),
        })
    }

    pub fn as_flat_instructions(&self) -> Result<Vec<Instruction>, anyhow::Error> {
        let mut reader = SliceReader::new(self.bytes);
        let mut instructions = vec![];

        while reader.has_more() {
            let program_counter = reader.position() as u64;
            let instruction = Instruction::parse(&mut reader, program_counter * 4)?;

            instructions.push(instruction);

            let instruction = &instructions[instructions.len() - 1];

            if let FormattedInstruction::SOPP(SOPPInstruction {
                op: SOPPOpCode::s_endpgm,
                ..
            }) = instruction.inner
            {
                break;
            };
        }

        Ok(instructions)
    }
}
