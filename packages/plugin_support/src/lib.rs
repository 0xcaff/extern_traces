#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]

mod io;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::slice;
use pm4::{convert, Command, PM4Packet, ShaderInvocation};

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "panic occurred at {}:{}: {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("panic occurred: {}", info.message());
    }

    core::intrinsics::abort();
}

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        libc::malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut libc::c_void)
    }
}

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;

#[no_mangle]
extern "C" fn log_pm4(buffer: *const u8, len: usize) {
    let buffer = unsafe { slice::from_raw_parts(buffer, len) };

    let buffer = PM4Packet::parse_all(buffer).unwrap();
    let (commands, _ignored_packets, _ignored_registers) = convert(&buffer).unwrap();
    for command in commands {
        match command {
            Command::Draw { pipeline, .. } => {
                if let Some(vertex_shader) = pipeline.vertex_shader.entrypoint_gpu_address {
                    let invocation =
                        unsafe { ShaderInvocation::decode_from_memory(vertex_shader) }.unwrap();

                    println!(
                        "vertex @ 0x{:x} len {}",
                        (vertex_shader as u64) << 8,
                        invocation.bytes.len() * 4
                    );
                }

                if let Some(pixel_shader) = pipeline.pixel_shader.address {
                    let invocation =
                        unsafe { ShaderInvocation::decode_from_memory(pixel_shader) }.unwrap();

                    println!(
                        "pixel @ 0x{:x} len {}",
                        (pixel_shader as u64) << 8,
                        invocation.bytes.len() * 4
                    );
                }
            }
            Command::Dispatch { pipeline, .. } => {
                let invocation =
                    unsafe { ShaderInvocation::decode_from_memory(pipeline.address_lo) }.unwrap();

                println!(
                    "compute @ 0x{:x} len {}",
                    (pipeline.address_lo as u64) << 8,
                    invocation.bytes.len() * 4
                );
            }
            _ => {}
        }
    }
}
