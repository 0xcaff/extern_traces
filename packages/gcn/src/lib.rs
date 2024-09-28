#![deny(warnings)]
#![no_std]
extern crate alloc;

pub mod instructions;
mod reader;
pub mod resources;
pub mod test_utils;

pub use reader::SliceReader;
