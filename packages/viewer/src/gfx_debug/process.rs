use crate::gfx_debug::compute::process_dispatch_command;
use crate::gfx_debug::ctx::GraphicsContext;
use gcn_extract::VertexBufferResourceWithRaw;
use pm4::{convert, Command, PM4Packet};
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

type CommandBufferBuilder = AutoCommandBufferBuilder<
    PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
    Arc<StandardCommandBufferAllocator>,
>;

// pub fn process_commands(graphics_context: &mut GraphicsContext, commands: &[PM4Packet]) {
//     let (commands, _, _) = convert(commands).unwrap();
//     for command in commands {
//         match command {
//             Command::Dispatch {
//                 dispatch_packet,
//                 pipeline,
//             } => process_dispatch_command(graphics_context, dispatch_packet, pipeline),
//             // Command::Draw {
//             //     draw_packet,
//             //     pipeline: pipeline_input,
//             // } => process_draw_command(graphics_context, draw_packet, pipeline_input),
//             _ => {}
//         }
//     }
// }

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
