#![feature(buf_read_has_data_left)]

use crate::instructions::Decoder;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

mod instructions;
mod reader;

fn main() -> Result<(), anyhow::Error> {
    let mut file =
        File::open("/Users/martin/projects/freegnm-examples/cube/assets/misc/cube.frag.sb")?;
    file.seek(SeekFrom::Start(120))?;
    let mut decoder = Decoder::new(file.take(76));

    loop {
        let instruction = match decoder.decode()? {
            Some(value) => value,
            None => break,
        };

        println!("{:#?}", instruction);
    }

    Ok(())
}
