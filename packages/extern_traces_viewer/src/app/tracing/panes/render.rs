use crate::gfx_debug::{process_commands, DebugHandle, ExtraData, GraphicsContext};
use anyhow::{format_err, Result};
use bytemuck;
use pm4::PM4Packet;
use std::io::Cursor;
use std::sync::Arc;

pub fn render_frame(
    ctx: &GraphicsContext,
    debug_handle: &mut DebugHandle,
    extra_data: &ExtraData,
) -> Result<Option<Arc<[u8]>>, anyhow::Error> {
    let draw_command_buffer = extra_data
        .draw_command_buffers
        .first()
        .ok_or_else(|| format_err!("missing first value"))?;
    let packets = PM4Packet::parse_all(draw_command_buffer)?;

    debug_handle.start_frame_capture();

    let known_shaders = extra_data
        .shaders
        .iter()
        .map(|it| (it.address, it))
        .collect();

    let (next_color, next_depth) = process_commands(
        ctx,
        packets.as_slice(),
        &extra_data.vertex_buffers,
        &extra_data.texture_buffers,
        known_shaders,
    )?;

    debug_handle.end_frame_capture();

    let Some(next_color) = next_color else {
        return Ok(None);
    };

    let mut encoded_writer = Cursor::new(Vec::new());

    {
        let mut encoder = png::Encoder::new(&mut encoded_writer, 1920, 1080);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&next_color)?;
    }

    Ok(Some(Arc::from(encoded_writer.into_inner())))
}
