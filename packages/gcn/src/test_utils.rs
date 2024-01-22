use crate::{DisplayInstruction, Instruction};
use std::io::Cursor;

pub struct GCNInstructionStream {
    pub instructions: Vec<Instruction>,
}

impl GCNInstructionStream {
    pub fn new(shader_bytes: &[u8]) -> Result<GCNInstructionStream, anyhow::Error> {
        let mut cursor = Cursor::new(shader_bytes);
        let mut instructions = vec![];

        loop {
            if cursor.is_empty() {
                break;
            }

            let position = cursor.position();
            let instruction = Instruction::parse(&mut cursor, position)?;
            instructions.push(instruction);
        }

        Ok(GCNInstructionStream { instructions })
    }

    pub fn debug(&self) -> String {
        format!("{:#?}", self.instructions)
    }

    pub fn displayed(&self) -> String {
        self.instructions
            .iter()
            .map(|it| {
                let display = it.inner.display();

                format!("{} {}", display.op, display.args.join(", "))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
