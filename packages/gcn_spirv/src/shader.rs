use anyhow::bail;
use core::slice;
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
        let bytes = self.bytes;
        let mut reader = SliceReader::new(self.bytes);
        let mut instructions = vec![];

        while reader.has_more() {
            let program_counter = unsafe { self.bytes.as_ptr().offset_from(bytes.as_ptr()) } as u64;
            let instruction = Instruction::parse(&mut reader, program_counter)?;

            instructions.push(instruction);
        }

        Ok(instructions)
    }
}
