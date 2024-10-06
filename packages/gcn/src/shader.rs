use crate::instructions::formats::{FormattedInstruction, SOPPInstruction};
use crate::instructions::ops::SOPPOpCode;
use crate::instructions::Instruction;
use crate::SliceReader;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};

pub struct GCNInstructionStream {
    pub instructions: Vec<Instruction>,
}

impl GCNInstructionStream {
    pub fn new(shader_bytes: &[u8]) -> Result<GCNInstructionStream, anyhow::Error> {
        let mut reader = SliceReader::new(bytemuck::cast_slice(shader_bytes));
        let mut instructions = vec![];

        loop {
            if !reader.has_more() {
                break;
            }

            let position = reader.position();
            let instruction = Instruction::parse(&mut reader, (position * 4) as _)?;
            let is_end_pgm = if let FormattedInstruction::SOPP(SOPPInstruction {
                op: SOPPOpCode::s_endpgm,
                ..
            }) = instruction.inner
            {
                true
            } else {
                false
            };

            instructions.push(instruction);
            if is_end_pgm {
                break;
            }
        }

        Ok(GCNInstructionStream { instructions })
    }

    pub fn debug(&self) -> String {
        format!("{:#?}", self.instructions)
    }

    pub fn displayed(&self) -> String {
        display_instructions(self.instructions.iter())
    }
}

pub fn display_instructions<'a>(instructions: impl Iterator<Item = &'a Instruction>) -> String {
    instructions
        .map(|it| format!("0x{:04x}: {}", it.program_counter, it.display().to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}
