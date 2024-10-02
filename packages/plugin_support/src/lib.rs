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
use anyhow::bail;
use bytemuck::{Pod, Zeroable};
use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::slice;
use gcn::instructions::formats::{FormattedInstruction, SOP1Instruction, SOPPInstruction};
use gcn::instructions::operands::{ScalarDestinationOperand, ScalarSourceOperand};
use gcn::instructions::ops::{SOP1OpCode, SOPPOpCode};
use gcn::instructions::Instruction;
use gcn::SliceReader;
use gcn_extract::{
    extract_buffer_usages, pixel_shader_extract_image_usages, SamplerResourceWithRaw,
    ShaderInvocation, TextureBufferResourceWithRaw, VertexBufferResourceWithRaw,
};
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

    (0usize..count)
        .into_iter()
        .map(|idx| {
            slice::from_raw_parts(
                draw_command_buffer_addrs[idx],
                draw_command_buffer_sizes[idx] as usize,
            )
        })
        .collect()
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
extern "C" fn sceGnmSubmitCommandBuffers_trace(
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

    let draw_command_buffers = unsafe { command_buffers(count as _, draw_buffers, draw_sizes) };

    let compute_command_buffers =
        unsafe { command_buffers(count as _, compute_buffers, compute_sizes) };

    trace_command_buffer_submit(
        thread_logging_state,
        &draw_command_buffers,
        &compute_command_buffers,
        thread_id,
        time,
        label_id,
    )
    .unwrap();
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

    let draw_command_buffers = unsafe { command_buffers(count as _, draw_buffers, draw_sizes) };

    let compute_command_buffers =
        unsafe { command_buffers(count as _, compute_buffers, compute_sizes) };

    trace_command_buffer_submit(
        thread_logging_state,
        &draw_command_buffers,
        &compute_command_buffers,
        thread_id,
        time,
        label_id,
    )
    .unwrap();
}

#[no_mangle]
extern "C" fn sceGnmSubmitAndFlipCommandBuffers_trace(
    args: *const Args,
    thread_logging_state: *mut ThreadLoggingState,
    time: u64,
    label_id: u64,
    thread_id: u64,
) {
    sceGnmSubmitCommandBuffers_trace(args, thread_logging_state, time, label_id, thread_id);
}

fn trace_command_buffer_submit(
    thread_logging_state: &mut ThreadLoggingState,
    draw_command_buffers: &[&[u8]],
    compute_command_buffers: &[&[u8]],
    thread_id: u64,
    time: u64,
    label_id: u64,
) -> Result<(), anyhow::Error> {
    let mut shaders = ShadersCollector::new();

    let mut total_size = size_of::<SpanStartAdditionalData>()
        + size_of::<u32>()
        + (draw_command_buffers.len() * size_of::<u32>())
        + (compute_command_buffers.len() * size_of::<u32>());

    for command_buffer in draw_command_buffers {
        total_size += command_buffer.len();
        shaders.collect(command_buffer)?;
    }

    for command_buffer in compute_command_buffers {
        total_size += command_buffer.len();
        shaders.collect(command_buffer)?;
    }

    total_size += size_of::<u32>(); // Shader count
    for (_, shader) in &shaders.shaders {
        total_size += size_of::<u32>() // address
            + size_of::<u8>()  // kind
            + size_of::<u32>() // length
            + shader.shader_bytes.len() * 4
        + size_of::<u32>() // vertex_buffer_references length
        + shader.vertex_buffer_references.len() * (size_of::<u32>() + size_of::<u32>())
    }

    total_size += size_of::<u32>();

    for vertex_buffer in &shaders.vertex_buffers {
        total_size +=
            (size_of::<u32>() * 4) + size_of::<u32>() + vertex_buffer.resource.len() as usize
    }

    let Some(mut res) = thread_logging_state.reserve(total_size) else {
        return Ok(());
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

        let len = shader.shader_bytes.len() as u32;
        res.write(bytemuck::cast_slice(&[len]));
        res.write(bytemuck::cast_slice(&shader.shader_bytes));

        res.write(bytemuck::cast_slice(&[
            shader.vertex_buffer_references.len() as u32,
        ]));
        for (program_counter, idx) in &shader.vertex_buffer_references {
            res.write(bytemuck::cast_slice(&[*program_counter as u32]));
            res.write(bytemuck::cast_slice(&[*idx as u32]));
        }
    }

    res.write(bytemuck::cast_slice(&[shaders.vertex_buffers.len() as u32]));
    for vertex_buffer in &shaders.vertex_buffers {
        res.write(bytemuck::cast_slice(&vertex_buffer.raw));

        res.write(bytemuck::cast_slice(&[vertex_buffer.resource.len() as u32]));
        res.write(unsafe { vertex_buffer.resource.bytes() });
    }

    // todo: detile and send texture buffers
    // res.write(bytemuck::cast_slice(&[shaders.texture_buffers.len() as u32]));
    // for texture_buffer in &shaders.texture_buffers {
    //     res.write(bytemuck::cast_slice(&texture_buffer.raw));
    //
    //     res.write(bytemuck::cast_slice(&[vertex_buffer.resource.len() as u32]));
    //     res.write(unsafe { vertex_buffer.resource.bytes() });
    // }

    thread_logging_state.flush(res);

    Ok(())
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
    has_fetch_shader: bool,
    shader_bytes: Vec<u32>,
    instructions: Vec<Instruction>,
    vertex_buffer_references: Vec<(u64, usize)>,
    texture_buffer_references: Vec<(u64, usize, SamplerResourceWithRaw)>,
    read_user_data: Vec<(u8, u32)>,
}

struct ShadersCollector {
    shaders: BTreeMap<u32, Shader>,
    vertex_buffers: Vec<VertexBufferResourceWithRaw>,
    texture_buffers: Vec<TextureBufferResourceWithRaw>,
}

impl ShadersCollector {
    pub fn new() -> ShadersCollector {
        ShadersCollector {
            shaders: BTreeMap::new(),
            vertex_buffers: vec![],
            texture_buffers: vec![],
        }
    }
}

impl ShadersCollector {
    pub fn collect(&mut self, buffer: &[u8]) -> Result<(), anyhow::Error> {
        let buffer = PM4Packet::parse_all(buffer)?;
        let (commands, _ignored_packets, _ignored_registers) = convert(&buffer)?;
        for command in commands {
            match command {
                Command::Draw { pipeline, .. } => {
                    if let Some(vertex_shader) = pipeline.vertex_shader.entrypoint_gpu_address {
                        self.collect_shader(
                            vertex_shader,
                            ShaderKind::Vertex,
                            &pipeline.vertex_shader.user_data.0,
                        )?;
                    }

                    if let Some(pixel_shader) = pipeline.pixel_shader.address {
                        self.collect_shader(
                            pixel_shader,
                            ShaderKind::Pixel,
                            &pipeline.pixel_shader.user_data.0,
                        )?;
                    }
                }
                Command::Dispatch { pipeline, .. } => {
                    self.collect_shader(
                        pipeline.address_lo,
                        ShaderKind::Compute,
                        &pipeline.user_data.0,
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn collect_shader(
        &mut self,
        shader_address: u32,
        kind: ShaderKind,
        user_data: &BTreeMap<u8, u32>,
    ) -> Result<(), anyhow::Error> {
        if shader_address == 0 {
            return Ok(());
        }

        if shader_address == 0xfe000f1 {
            // this is an embedded vertex shader, it seems to be in a protected memory region by
            // default. skip it as it uses rect list which requires geometry shader to emulate
            // its probably worth shipping this information to the client rendering pipeline somehow
            return Ok(());
        }

        let item = match self.shaders.entry(shader_address) {
            Entry::Vacant(item) => item,
            Entry::Occupied(item) => {
                let value = item.get();
                if value.has_fetch_shader {
                    for (key, value) in &value.read_user_data {
                        if user_data.get(key) != Some(value) {
                            bail!("not allowed")
                        }
                    }

                    return Ok(());
                }

                return Ok(());
            }
        };

        let shader = unsafe { shader_extract(shader_address, user_data)? };

        let instructions = shader.instructions;

        let buffer_usages = extract_buffer_usages(instructions.as_slice(), user_data);
        let mut vertex_buffer_references = vec![];

        for vertex_buffer_usage in &buffer_usages {
            let idx = (|| {
                if let Some(idx) = self
                    .vertex_buffers
                    .iter()
                    .enumerate()
                    .filter(|(idx, it)| it.resource == vertex_buffer_usage.resource.resource)
                    .map(|(idx, _)| idx)
                    .next()
                {
                    return idx;
                };

                self.vertex_buffers
                    .push(vertex_buffer_usage.resource.clone());
                self.vertex_buffers.len() - 1
            })();

            vertex_buffer_references.push((vertex_buffer_usage.program_counter, idx));
        }

        let texture_usages = pixel_shader_extract_image_usages(instructions.as_slice(), user_data);

        let mut texture_buffer_references = vec![];

        for texture_buffer_usage in &texture_usages {
            let idx = (|| {
                if let Some(idx) = self
                    .texture_buffers
                    .iter()
                    .enumerate()
                    .filter(|(idx, it)| {
                        it.resource == texture_buffer_usage.texture_resource.resource
                    })
                    .map(|(idx, _)| idx)
                    .next()
                {
                    return idx;
                };

                self.texture_buffers
                    .push(texture_buffer_usage.texture_resource.clone());
                self.texture_buffers.len() - 1
            })();

            texture_buffer_references.push((
                texture_buffer_usage.program_counter,
                idx,
                texture_buffer_usage.sampler_resource.clone(),
            ));
        }

        item.insert(Shader {
            shader_bytes: shader.raw,
            has_fetch_shader: shader.has_fetch_shader,
            read_user_data: shader.read_user_data,
            instructions,
            kind,
            vertex_buffer_references,
            texture_buffer_references,
        });

        Ok(())
    }
}

struct ExtractedShader {
    raw: Vec<u32>,
    instructions: Vec<Instruction>,
    has_fetch_shader: bool,
    read_user_data: Vec<(u8, u32)>,
}

unsafe fn shader_extract(
    shader_address: u32,
    user_data: &BTreeMap<u8, u32>,
) -> Result<ExtractedShader, anyhow::Error> {
    let entry_point = (shader_address as u64) << 8;
    let original_shader_bytes = slice::from_raw_parts(entry_point as *const u32, usize::MAX);

    let mut stitched_shader_bytes = Vec::new();
    let mut instructions = vec![];
    let mut reader = SliceReader::new(original_shader_bytes);
    let mut has_fetch_shader = false;
    let mut read_user_data = Vec::new();
    let mut max_position = None;

    loop {
        let position = reader.position();
        if let Some(max_position) = max_position {
            if position >= max_position {
                break;
            }
        }

        let program_counter = stitched_shader_bytes.len() as u64 * 4;

        let instruction = Instruction::parse(&mut reader, program_counter)?;

        if stitched_shader_bytes.is_empty() {
            if let Instruction {
                inner:
                    FormattedInstruction::SOP1(SOP1Instruction {
                        op: SOP1OpCode::s_mov_b32,
                        sdst: ScalarDestinationOperand::VccHi,
                        ssrc0: ScalarSourceOperand::LiteralConstant,
                    }),
                literal_constant: Some(constant),
                ..
            } = instruction
            {
                max_position = Some((constant as usize + 1) * 2)
            }
        }

        let fetch_shader_branch = (|| -> Option<u64> {
            let FormattedInstruction::SOP1(SOP1Instruction {
                op: SOP1OpCode::s_swappc_b64,
                ssrc0:
                    ScalarSourceOperand::Destination(ScalarDestinationOperand::ScalarGPR(sgpr_base)),
                ..
            }) = instruction.inner
            else {
                return None;
            };

            let lo = *user_data.get(&sgpr_base)?;
            let hi = *user_data.get(&(sgpr_base + 1))?;

            let address = (hi as u64) << 32 | lo as u64;

            read_user_data.push((sgpr_base, lo));
            read_user_data.push((sgpr_base + 1, hi));

            Some(address)
        })();

        match fetch_shader_branch {
            Some(address) => {
                has_fetch_shader = true;

                let fetch_shader_dwords = address as *const u32;
                let fetch_shader_bytes_unsafe =
                    slice::from_raw_parts(fetch_shader_dwords, usize::MAX);

                let mut fetch_shader_slice_reader_unsafe =
                    SliceReader::new(fetch_shader_bytes_unsafe);

                loop {
                    let fetch_shader_position = fetch_shader_slice_reader_unsafe.position();
                    let program_counter = stitched_shader_bytes.len() as u64 * 4;

                    let instruction =
                        Instruction::parse(&mut fetch_shader_slice_reader_unsafe, program_counter)?;

                    if let FormattedInstruction::SOP1(SOP1Instruction {
                        op: SOP1OpCode::s_setpc_b64,
                        ..
                    }) = &instruction.inner
                    {
                        break;
                    }

                    let fetch_shader_position_end = fetch_shader_slice_reader_unsafe.position();

                    instructions.push(instruction);
                    stitched_shader_bytes.extend_from_slice(
                        &fetch_shader_bytes_unsafe
                            [fetch_shader_position..fetch_shader_position_end],
                    );
                }
            }
            None => {
                let position_end = reader.position();
                instructions.push(instruction);
                stitched_shader_bytes
                    .extend_from_slice(&original_shader_bytes[position..position_end]);

                let instruction = &instructions[instructions.len() - 1];

                if let FormattedInstruction::SOPP(SOPPInstruction {
                    op: SOPPOpCode::s_endpgm,
                    ..
                }) = &instruction.inner
                {
                    break;
                };
            }
        };
    }

    Ok(ExtractedShader {
        instructions,
        raw: stitched_shader_bytes,
        has_fetch_shader,
        read_user_data,
    })
}
