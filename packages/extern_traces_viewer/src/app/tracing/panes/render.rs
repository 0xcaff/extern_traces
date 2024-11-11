use crate::gfx_debug::{process_commands, DebugHandle, ExtraData, GraphicsContext};
use anyhow::{format_err, Result};
use bytemuck;
use eframe::egui::ColorImage;
use pm4::PM4Packet;

pub fn render_frame(
    ctx: &GraphicsContext,
    debug_handle: &mut DebugHandle,
    extra_data: &ExtraData,
) -> Result<Option<ColorImage>, anyhow::Error> {
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

    let image = ColorImage::from_rgba_unmultiplied([1920, 1080], next_color.as_ref());

    Ok(Some(image))
}
