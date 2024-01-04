use crate::instructions::Decoder;
use crate::reader::ReadError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

mod bitrange;
mod instructions;
mod pm4;
mod reader;

fn main() -> Result<(), anyhow::Error> {
    let mut file =
        File::open("/Users/martin/projects/freegnm-examples/cube/assets/misc/cube.frag.sb")?;
    file.seek(SeekFrom::Start(120))?;
    let mut decoder = Decoder::new(file.take(76));

    loop {
        let instruction = match decoder.decode() {
            Ok(value) => value,
            Err(ReadError::Eof) => break,
            Err(ReadError::Error(error)) => {
                println!("{:?}", error);
                continue;
            }
        };

        println!("{:#?}", instruction);
    }

    Ok(())
}
