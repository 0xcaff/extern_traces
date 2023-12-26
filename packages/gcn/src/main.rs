use crate::instructions::{Decoder, GfxLevel, InstructionParseErrorKind};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

mod instructions;

fn main() -> Result<(), anyhow::Error> {
    let mut file =
        File::open("/Users/martin/projects/freegnm-examples/cube/assets/misc/cube.frag.sb")?;
    file.seek(SeekFrom::Start(120))?;
    let mut decoder = Decoder::new(GfxLevel::GFX7, file.take(76));

    loop {
        let instruction = match decoder.decode() {
            Ok(value) => value,
            Err(InstructionParseErrorKind::Eof) => break,
            Err(InstructionParseErrorKind::Error(error)) => {
                println!("{:?}", error);
                continue;
            }
        };

        println!("{:?}", instruction);
    }

    Ok(())
}
