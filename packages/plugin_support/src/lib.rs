#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(ptr_as_ref_unchecked)]
extern crate alloc;

mod ffi;
mod io;

use alloc::collections::btree_map::Entry;
use crate::ffi::{Args, ThreadLoggingState};
use alloc::collections::BTreeMap;
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

unsafe fn command_buffers<'a>(
    count: usize,
    addrs: *const *const u8,
    sizes: *const u32,
) -> impl Iterator<Item = &'a [u8]> {
    let draw_command_buffer_addrs = slice::from_raw_parts(addrs, count);
    let draw_command_buffer_sizes = slice::from_raw_parts(sizes, count);

    (0usize..count).into_iter().map(|idx| {
        slice::from_raw_parts(
            draw_command_buffer_addrs[idx],
            draw_command_buffer_sizes[idx] as usize,
        )
    })
}

#[no_mangle]
extern "C" fn sceGnmSubmitAndFlipCommandBuffersForWorkload_trace(
    args: *const Args,
    thread_logging_state: *mut ThreadLoggingState,
) {
    let args = unsafe { args.as_ref_unchecked() };
    let thread_logging_state = unsafe { thread_logging_state.as_mut_unchecked() };

    let count = args.args[1] as u32;
    let draw_buffers = args.args[2] as *const *const u8;
    let draw_sizes = args.args[3] as *const u32;
    let compute_buffers = args.args[4] as *const *const u8;
    let compute_sizes = args.args[5] as *const u32;

    let mut shaders = ShadersCollector::new();
    
    let draw_command_buffers = unsafe { command_buffers(count as _, draw_buffers, draw_sizes) };
    for command_buffer in draw_command_buffers {
        shaders.collect(command_buffer);
    }
    
    let compute_command_buffers =
        unsafe { command_buffers(count as _, compute_buffers, compute_sizes) };
    for command_buffer in compute_command_buffers {
        shaders.collect(command_buffer);
    }
    
    
}

enum ShaderKind {
    Compute,
    Vertex,
    Pixel,
}

struct Shader {
    kind: ShaderKind,
    shader_invocation: ShaderInvocation,
}

struct ShadersCollector {
    shaders: BTreeMap<u32, Shader>,
}

impl ShadersCollector {
    pub fn new() -> ShadersCollector {
        ShadersCollector {
            shaders: BTreeMap::new()
        }
    }
}

impl ShadersCollector {
    pub fn collect(&mut self, buffer: &[u8]) {
        let buffer = PM4Packet::parse_all(buffer).unwrap();
        let (commands, _ignored_packets, _ignored_registers) = convert(&buffer).unwrap();
        for command in commands {
            match command {
                Command::Draw { pipeline, .. } => {
                    if let Some(vertex_shader) = pipeline.vertex_shader.entrypoint_gpu_address {
                        self.collect_shader(vertex_shader, ShaderKind::Vertex);
                    }

                    if let Some(pixel_shader) = pipeline.pixel_shader.address {
                        self.collect_shader(pixel_shader, ShaderKind::Pixel);
                    }
                }
                Command::Dispatch { pipeline, .. } => {
                    self.collect_shader(pipeline.address_lo, ShaderKind::Compute);
                }
                _ => {}
            }
        }
    }

    fn collect_shader(&mut self, shader_address: u32, kind: ShaderKind) {
        if shader_address == 0 {
            return;
        }

        let entry = self.shaders.entry(shader_address);
        let Entry::Vacant(item) = entry else {
            return;
        };

        let Ok(shader_invocation) = (unsafe { ShaderInvocation::decode_from_memory(shader_address) }) else {
            return;
        };

        item.insert(
            Shader {
                shader_invocation,
                kind,
            }
        );
    }
}
