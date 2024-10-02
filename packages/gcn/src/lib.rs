#![deny(warnings)]
#![no_std]
extern crate alloc;

pub mod instructions;
mod reader;
pub mod resources;
mod shader;

pub use reader::SliceReader;
pub use shader::GCNInstructionStream;
