#![no_std]
#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(ptr_as_ref_unchecked)]
extern crate alloc;

mod ffi;
mod io;

use crate::ffi::{Args, ThreadLoggingState};
use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use bytemuck::{Pod, Zeroable};
use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::slice;
use gcn::instructions::Instruction;
use gcn_extract::{extract_buffer_usages, pixel_shader_extract_image_usages, ShaderInvocation};
use pm4::{convert, Command, PM4Packet};

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
) -> Vec<&'a [u8]> {
    if addrs.is_null() {
        return vec![&[]; count];
    }

    if sizes.is_null() {
        return vec![&[]; count];
    }

    let draw_command_buffer_addrs = slice::from_raw_parts(addrs, count);
    let draw_command_buffer_sizes = slice::from_raw_parts(sizes, count);

    (0usize..count).into_iter().map(|idx| {
        slice::from_raw_parts(
            draw_command_buffer_addrs[idx],
            draw_command_buffer_sizes[idx] as usize,
        )
    }).collect()
}

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
struct SpanStartAdditionalData {
    message_tag: u64,
    thread_id: u64,
    time: u64,
    label_id: u64,
    extra_data_length: u64,
}

#[no_mangle]
extern "C" fn sceGnmSubmitAndFlipCommandBuffersForWorkload_trace(
    args: *const Args,
    thread_logging_state: *mut ThreadLoggingState,
    time: u64,
    label_id: u64,
    thread_id: u64,
) {
    let args = unsafe { args.as_ref_unchecked() };
    let thread_logging_state = unsafe { thread_logging_state.as_mut_unchecked() };

    let count = args.args[1] as u32;
    let draw_buffers = args.args[2] as *const *const u8;
    let draw_sizes = args.args[3] as *const u32;
    let compute_buffers = args.args[4] as *const *const u8;
    let compute_sizes = args.args[5] as *const u32;

    let draw_command_buffers =
        unsafe { command_buffers(count as _, draw_buffers, draw_sizes) };

    let compute_command_buffers =
        unsafe { command_buffers(count as _, compute_buffers, compute_sizes) };

    trace_command_buffer_submit(
        thread_logging_state,
        &draw_command_buffers,
        &compute_command_buffers,
        thread_id,
        time,
        label_id,
    );
}

#[no_mangle]
extern "C" fn sceGnmSubmitAndFlipCommandBuffers_trace(
    args: *const Args,
    thread_logging_state: *mut ThreadLoggingState,
    time: u64,
    label_id: u64,
    thread_id: u64,
) {
    let args = unsafe { args.as_ref_unchecked() };
    let thread_logging_state = unsafe { thread_logging_state.as_mut_unchecked() };

    let count = args.args[0] as u32;
    let draw_buffers = args.args[1] as *const *const u8;
    let draw_sizes = args.args[2] as *const u32;
    let compute_buffers = args.args[3] as *const *const u8;
    let compute_sizes = args.args[4] as *const u32;

    let draw_command_buffers =
        unsafe { command_buffers(count as _, draw_buffers, draw_sizes) };

    let compute_command_buffers =
        unsafe { command_buffers(count as _, compute_buffers, compute_sizes) };

    trace_command_buffer_submit(
        thread_logging_state,
        &draw_command_buffers,
        &compute_command_buffers,
        thread_id,
        time,
        label_id,
    );
}

fn trace_command_buffer_submit(
    thread_logging_state: &mut ThreadLoggingState,
    draw_command_buffers: &[&[u8]],
    compute_command_buffers: &[&[u8]],
    thread_id: u64,
    time: u64,
    label_id: u64,
) {
    let mut shaders = ShadersCollector::new();

    let mut total_size = size_of::<SpanStartAdditionalData>()
        + size_of::<u32>()
        + (draw_command_buffers.len() * size_of::<u32>())
        + (compute_command_buffers.len() * size_of::<u32>());

    for command_buffer in draw_command_buffers {
        total_size += command_buffer.len();
        shaders.collect(command_buffer);
    }

    for command_buffer in compute_command_buffers {
        total_size += command_buffer.len();
        shaders.collect(command_buffer);
    }

    total_size += size_of::<u32>(); // Shader count
    for (_, shader) in &shaders.shaders {
        total_size += size_of::<u32>() // address
            + size_of::<u8>()  // kind
            + size_of::<u32>() // length
            + shader.shader_invocation.bytes.len() * 4;
    }

    let Some(mut res) = thread_logging_state.reserve(total_size) else {
        return;
    };

    let span_header = SpanStartAdditionalData {
        message_tag: 3,
        thread_id,
        time,
        label_id,
        extra_data_length: total_size as u64 - size_of::<SpanStartAdditionalData>() as u64,
    };

    res.write(bytemuck::cast_slice(&[span_header]));

    let count = draw_command_buffers.len() as u32;
    res.write(bytemuck::cast_slice(&[count]));

    for draw_command_buffer in draw_command_buffers {
        let value = draw_command_buffer.len() as u32;
        res.write(bytemuck::cast_slice(&[value]));
    }

    for compute_command_buffer in compute_command_buffers {
        let value = compute_command_buffer.len() as u32;
        res.write(bytemuck::cast_slice(&[value]));
    }

    for draw_command_buffer in draw_command_buffers {
        res.write(draw_command_buffer);
    }

    for compute_command_buffer in compute_command_buffers {
        res.write(compute_command_buffer);
    }

    res.write(bytemuck::cast_slice(&[shaders.shaders.len() as u32]));

    for (&address, shader) in &shaders.shaders {
        res.write(bytemuck::cast_slice(&[address]));

        let kind_u8 = shader.kind as u8;
        res.write(bytemuck::cast_slice(&[kind_u8]));

        let len = shader.shader_invocation.bytes.len() as u32;
        res.write(bytemuck::cast_slice(&[len]));

        res.write(bytemuck::cast_slice(shader.shader_invocation.bytes));
    }

    thread_logging_state.flush(res);
    println!("done!");
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum ShaderKind {
    Compute,
    Vertex,
    Pixel,
}

struct Shader {
    kind: ShaderKind,
    shader_invocation: ShaderInvocation,
    instructions: Option<Vec<Instruction>>,
}

struct ShadersCollector {
    shaders: BTreeMap<u32, Shader>,
}

impl ShadersCollector {
    pub fn new() -> ShadersCollector {
        ShadersCollector {
            shaders: BTreeMap::new(),
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
                        self.collect_shader(
                            vertex_shader,
                            ShaderKind::Vertex,
                            &pipeline.vertex_shader.user_data.0,
                        );
                    }

                    if let Some(pixel_shader) = pipeline.pixel_shader.address {
                        self.collect_shader(
                            pixel_shader,
                            ShaderKind::Pixel,
                            &pipeline.pixel_shader.user_data.0,
                        );
                    }
                }
                Command::Dispatch { pipeline, .. } => {
                    self.collect_shader(
                        pipeline.address_lo,
                        ShaderKind::Compute,
                        &pipeline.user_data.0,
                    );
                }
                _ => {}
            }
        }
    }

    fn collect_shader(
        &mut self,
        shader_address: u32,
        kind: ShaderKind,
        user_data: &BTreeMap<u8, u32>,
    ) {
        if shader_address == 0 {
            return;
        }

        let shader = match self.shaders.entry(shader_address) {
            Entry::Vacant(item) => {
                let Ok(shader_invocation) =
                    (unsafe { ShaderInvocation::decode_from_memory(shader_address) })
                else {
                    return;
                };

                let instructions = shader_invocation.as_flat_instructions();

                if let Err(err) = &instructions {
                    println!("extern_traces: {:?}", err);
                }

                item.insert(Shader {
                    shader_invocation,
                    kind,
                    instructions: instructions.ok(),
                })
            }
            Entry::Occupied(value) => value.into_mut(),
        };

        if let Some(instructions) = shader.instructions.as_ref() {
            let buffer_usages = extract_buffer_usages(instructions.as_slice(), user_data);
            let texture_usages =
                pixel_shader_extract_image_usages(instructions.as_slice(), user_data);

            println!(
                "extern_traces: {} {:#?} {:#?} {:#?}",
                shader_address, kind, buffer_usages, texture_usages
            );
        }
    }
}
