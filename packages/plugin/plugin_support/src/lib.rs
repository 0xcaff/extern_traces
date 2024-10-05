#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(ptr_as_ref_unchecked)]
#![feature(naked_functions)]

extern crate alloc;
mod hook;
mod logger;
mod mapped_memory;
mod platform;
