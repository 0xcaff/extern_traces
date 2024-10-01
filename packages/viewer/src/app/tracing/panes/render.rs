use crate::gfx_debug::{process_commands, ExtraData, GraphicsContext};
use anyhow::{format_err, Result};
use bytemuck;
use pm4::PM4Packet;
use std::sync::Arc;

pub fn render_frame(bytes: &[u8]) -> Result<Option<Arc<[u8]>>, anyhow::Error> {
    let extra_data = ExtraData::parse(bytes)?;
    let draw_command_buffer = extra_data
        .draw_command_buffers
        .first()
        .ok_or_else(|| format_err!("missing first value"))?;
    let packets = PM4Packet::parse_all(draw_command_buffer)?;

    let (mut ctx, debug_handle) = GraphicsContext::init();

    let known_shaders = extra_data
        .shaders
        .into_iter()
        .map(|it| (it.address, it))
        .collect();

    let (next_color, next_depth) = process_commands(
        &mut ctx,
        packets.as_slice(),
        &extra_data.vertex_buffers,
        known_shaders,
    )?;

    Ok(next_color.map(Arc::from))
}
