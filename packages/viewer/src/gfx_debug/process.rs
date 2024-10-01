use crate::gfx_debug::compute::process_dispatch_command;
use crate::gfx_debug::ctx::GraphicsContext;
use crate::gfx_debug::draw::process_draw_command;
use crate::gfx_debug::resources::buffers::BuffersDataContainer;
use gcn_extract::VertexBufferResourceWithRaw;
use pm4::{convert, Command, PM4Packet};
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Seek};
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

type CommandBufferBuilder = AutoCommandBufferBuilder<
    PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
    Arc<StandardCommandBufferAllocator>,
>;

pub fn process_commands(
    graphics_context: &mut GraphicsContext,
    commands: &[PM4Packet],
    initial_vertex_buffers: &[VertexBuffer],
    known_shaders: BTreeMap<u32, EncodedShader>,
) -> Result<(Option<Box<[u8]>>, Option<Box<[u8]>>), anyhow::Error> {
    let mut data = BuffersDataContainer::new(initial_vertex_buffers);
    let mut color_buffer = None;
    let mut depth_buffer = None;

    let (commands, _, _) = convert(commands)?;

    for command in commands {
        match command {
            // Command::Dispatch {
            //     dispatch_packet,
            //     pipeline,
            // } => {
            //     let result = process_dispatch_command(
            //         graphics_context,
            //         dispatch_packet,
            //         pipeline,
            //         initial_vertex_buffers,
            //         &data,
            //         &known_shaders,
            //     )?;
            //     data.update(result);
            // }
            Command::Draw {
                draw_packet,
                pipeline: pipeline_input,
            } => {
                let result = process_draw_command(
                    graphics_context,
                    draw_packet,
                    pipeline_input,
                    initial_vertex_buffers,
                    &data,
                    &known_shaders,
                    &color_buffer,
                    &depth_buffer,
                )?;
                data.update(result.buffers);

                color_buffer = result.color_buffer;
                depth_buffer = result.depth_buffer;
            }
            _ => {}
        }
    }

    Ok((color_buffer, depth_buffer))
}

pub struct ExtraData<'a> {
    pub draw_command_buffers: Vec<&'a [u8]>,
    pub compute_command_buffers: Vec<&'a [u8]>,
    pub shaders: Vec<EncodedShader<'a>>,
    pub vertex_buffers: Vec<VertexBuffer<'a>>,
}

pub struct EncodedShader<'a> {
    pub address: u32,
    pub kind: ShaderKind,
    pub bytes: &'a [u8],
    pub vertex_buffer_references: Vec<(u64, usize)>,
}

pub struct VertexBuffer<'a> {
    pub vertex_buffer: VertexBufferResourceWithRaw,
    pub bytes: &'a [u8],
}

pub enum ShaderKind {
    Compute,
    Vertex,
    Pixel,
}

impl From<u8> for ShaderKind {
    fn from(value: u8) -> Self {
        match value {
            0 => ShaderKind::Compute,
            1 => ShaderKind::Vertex,
            2 => ShaderKind::Pixel,
            _ => panic!("Invalid shader kind: {}", value),
        }
    }
}

impl ExtraData<'_> {
    pub fn parse(data: &[u8]) -> anyhow::Result<ExtraData> {
        let mut cursor = Cursor::new(data);

        // Read count
        let mut count_bytes = [0u8; size_of::<u32>()];
        cursor.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes);

        // Read draw and compute sizes
        let mut draw_sizes = vec![0u32; count as usize];
        let mut compute_sizes = vec![0u32; count as usize];
        cursor.read_exact(bytemuck::cast_slice_mut(&mut draw_sizes))?;
        cursor.read_exact(bytemuck::cast_slice_mut(&mut compute_sizes))?;

        // Read command buffers
        let mut draw_command_buffers = Vec::with_capacity(count as usize);
        let mut compute_command_buffers = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let draw_start = cursor.position() as usize;
            cursor.seek(std::io::SeekFrom::Current(draw_sizes[i] as i64))?;
            draw_command_buffers.push(&data[draw_start..draw_start + draw_sizes[i] as usize]);
        }

        for i in 0..count as usize {
            let compute_start = cursor.position() as usize;
            cursor.seek(std::io::SeekFrom::Current(compute_sizes[i] as i64))?;
            compute_command_buffers
                .push(&data[compute_start..compute_start + compute_sizes[i] as usize]);
        }

        // Read shader count
        let mut shader_count_bytes = [0u8; size_of::<u32>()];
        cursor.read_exact(&mut shader_count_bytes)?;
        let shader_count = u32::from_le_bytes(shader_count_bytes);

        // Read shaders
        let mut shaders = Vec::with_capacity(shader_count as usize);
        for _ in 0..shader_count {
            let mut address_bytes = [0u8; size_of::<u32>()];
            cursor.read_exact(&mut address_bytes)?;
            let address = u32::from_le_bytes(address_bytes);

            let mut kind_byte = [0u8; 1];
            cursor.read_exact(&mut kind_byte)?;
            let kind = ShaderKind::from(kind_byte[0]);

            let mut length_bytes = [0u8; size_of::<u32>()];
            cursor.read_exact(&mut length_bytes)?;
            let length = u32::from_le_bytes(length_bytes) * 4;

            let start = cursor.position() as usize;
            cursor.seek(std::io::SeekFrom::Current(length as i64))?;
            let bytes = &data[start..start + length as usize];

            // Read vertex buffer references
            let mut vbr_count_bytes = [0u8; size_of::<u32>()];
            cursor.read_exact(&mut vbr_count_bytes)?;
            let vbr_count = u32::from_le_bytes(vbr_count_bytes);

            let mut vertex_buffer_references = Vec::with_capacity(vbr_count as usize);
            for _ in 0..vbr_count {
                let mut pc_bytes = [0u8; size_of::<u32>()];
                cursor.read_exact(&mut pc_bytes)?;
                let program_counter = u64::from(u32::from_le_bytes(pc_bytes));

                let mut idx_bytes = [0u8; size_of::<u32>()];
                cursor.read_exact(&mut idx_bytes)?;
                let idx = u32::from_le_bytes(idx_bytes) as usize;

                vertex_buffer_references.push((program_counter, idx));
            }

            shaders.push(EncodedShader {
                address,
                kind,
                bytes,
                vertex_buffer_references,
            });
        }

        // Read vertex buffer count
        let mut vb_count_bytes = [0u8; size_of::<u32>()];
        cursor.read_exact(&mut vb_count_bytes)?;
        let vb_count = u32::from_le_bytes(vb_count_bytes);

        // Read vertex buffers
        let mut vertex_buffers = Vec::with_capacity(vb_count as usize);
        for _ in 0..vb_count {
            let mut raw_values = [0u32; 4];
            cursor.read_exact(bytemuck::cast_slice_mut(&mut raw_values))?;
            let vertex_buffer = VertexBufferResourceWithRaw::from_bits(&raw_values);

            let mut length_bytes = [0u8; size_of::<u32>()];
            cursor.read_exact(&mut length_bytes)?;
            let length = u32::from_le_bytes(length_bytes) as usize;

            let start = cursor.position() as usize;
            cursor.seek(std::io::SeekFrom::Current(length as i64))?;
            let bytes = &data[start..start + length as usize];

            vertex_buffers.push(VertexBuffer {
                vertex_buffer,
                bytes,
            });
        }

        Ok(ExtraData {
            draw_command_buffers,
            compute_command_buffers,
            shaders,
            vertex_buffers,
        })
    }
}
