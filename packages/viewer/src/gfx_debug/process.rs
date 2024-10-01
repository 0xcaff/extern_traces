use crate::gfx_debug::compute::process_dispatch_command;
use crate::gfx_debug::ctx::GraphicsContext;
use crate::gfx_debug::draw::process_draw_command;
use crate::gfx_debug::resources::buffers::BuffersDataContainer;
use gcn_extract::VertexBufferResourceWithRaw;
use pm4::{convert, Command, PM4Packet};
use std::collections::BTreeMap;
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
            Command::Dispatch {
                dispatch_packet,
                pipeline,
            } => {
                let result = process_dispatch_command(
                    graphics_context,
                    dispatch_packet,
                    pipeline,
                    initial_vertex_buffers,
                    &data,
                    &known_shaders,
                )?;
                data.update(result);
            }
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

enum ShaderKind {
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

pub struct EncodedShader<'a> {
    pub address: u32,
    pub kind: ShaderKind,
    pub bytes: &'a [u8],
    pub vertex_buffer_references: Vec<(u64, usize)>,
}

pub struct ExtraData<'a> {
    pub draw_command_buffers: Vec<&'a [u8]>,
    pub compute_command_buffers: Vec<&'a [u8]>,
    pub shaders: Vec<EncodedShader<'a>>,
    pub vertex_buffers: Vec<VertexBuffer<'a>>,
}

pub struct VertexBuffer<'a> {
    pub vertex_buffer: VertexBufferResourceWithRaw,
    pub bytes: &'a [u8],
}
