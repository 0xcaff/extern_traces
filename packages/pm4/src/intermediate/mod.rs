pub mod build;

use crate::intermediate::build::{Build, Builder, Finalize, Initialize};
use crate::{
    DrawIndexAutoPacket, EndOfPipePacket, PM4Packet, RegisterEntry, Type3Packet, Type3PacketValue,
    CB_COLOR0_ATTRIB, CB_COLOR0_CMASK_SLICE, CB_COLOR0_INFO, CB_COLOR0_PITCH, CB_COLOR0_SLICE,
    CB_COLOR0_VIEW, CB_SHADER_MASK, CB_TARGET_MASK, DB_DEPTH_CONTROL, DB_DEPTH_INFO, DB_DEPTH_SIZE,
    DB_DEPTH_SLICE, DB_DEPTH_VIEW, DB_HTILE_SURFACE, DB_RENDER_CONTROL, DB_SHADER_CONTROL,
    DB_STENCIL_INFO, DB_Z_INFO, PA_CL_VS_OUT_CNTL, PA_CL_VTE_CNTL, PA_SC_SCREEN_SCISSOR_BR,
    PA_SC_SCREEN_SCISSOR_TL, PA_SU_HARDWARE_SCREEN_OFFSET, PA_SU_SC_MODE_CNTL, SPI_BARYC_CNTL,
    SPI_PS_INPUT_CNTL_0, SPI_PS_INPUT_ENA, SPI_PS_IN_CONTROL, SPI_SHADER_COL_FORMAT,
    SPI_SHADER_PGM_RSRC1_PS, SPI_SHADER_PGM_RSRC1_VS, SPI_SHADER_PGM_RSRC2_PS,
    SPI_SHADER_PGM_RSRC2_VS, SPI_SHADER_POS_FORMAT, SPI_SHADER_Z_FORMAT, SPI_VS_OUT_CONFIG,
};
use pm4_internal_macros::{Build, BuildUserData};

#[derive(Debug)]
pub enum Command {
    Draw {
        draw_packet: DrawIndexAutoPacket,
        pipeline: GraphicsPipeline,
    },
    EndOfPipe(EndOfPipePacket),
}

pub fn convert(commands: &[PM4Packet]) -> Vec<Command> {
    let mut result = vec![];

    let mut builder = <GraphicsPipeline as Build<RegisterEntry>>::Builder::new();

    for packet in commands {
        match packet {
            PM4Packet::Type3(Type3Packet {
                header,
                value: Type3PacketValue::SetContextRegister { values },
            })
            | PM4Packet::Type3(Type3Packet {
                header,
                value: Type3PacketValue::SetShaderRegister { values },
            }) => {
                for entry in values {
                    let Some(entry) = entry else {
                        continue;
                    };

                    let accepted = builder.update(entry);
                    if let None = accepted {
                        // todo: log
                    }
                }
            }
            PM4Packet::Type3(Type3Packet {
                header,
                value: Type3PacketValue::DrawIndexAuto(draw_packet),
            }) => {
                result.push(Command::Draw {
                    draw_packet: draw_packet.clone(),
                    pipeline: builder.clone().finalize().unwrap(),
                });
            }
            PM4Packet::Type3(Type3Packet {
                header,
                value: Type3PacketValue::EndOfPipe(end_of_pipe),
            }) => result.push(Command::EndOfPipe(end_of_pipe.clone())),
            _ => {}
        }
    }

    result
}

/// A structured intermediate representation of data in pm4 graphics pipeline.
///
/// The goal with this is:
/// * Validate structural assumptions about the input stream like the pipeline
///   will always include an DB_DEPT_CONTROL entry or a SPI_SHADER_PGM_LO_PS
///   will always be accompanied with a SPI_SHADER_PGM_RSRC1_PS entry.
///
/// * Express as complete a set of possible information available to downstream
///   code to get a sense of implementation completeness. Things are grouped
///   according to their names (things with the same prefixes live near each
///   other).
///
/// * Provide a representation optimized for later stages to read.
#[derive(Build, Debug)]
#[entry(RegisterEntry)]
pub struct GraphicsPipeline {
    depth_buffer: DepthBuffer,

    primitive_assembly: PrimitiveAssembly,

    #[entry(RegisterEntry::CB_TARGET_MASK)]
    target_mask: CB_TARGET_MASK,

    #[entry(RegisterEntry::CB_SHADER_MASK)]
    shader_mask: CB_SHADER_MASK,

    pub color0: ColorBuffer,

    shader: Shader,
    pub pixel_shader: PixelShader,
    pub vertex_shader: Option<VertexShader>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct DepthBuffer {
    stencil: Stencil,
    depth: Depth,
    z: Z,

    #[entry(RegisterEntry::DB_RENDER_CONTROL)]
    render_control: DB_RENDER_CONTROL,

    #[entry(RegisterEntry::DB_SHADER_CONTROL)]
    shader_control: DB_SHADER_CONTROL,

    htile: Option<HTile>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct HTile {
    #[entry(RegisterEntry::DB_HTILE_DATA_BASE)]
    hitle_data_base: u32,

    #[entry(RegisterEntry::DB_HTILE_SURFACE)]
    htile_surface: DB_HTILE_SURFACE,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct Depth {
    #[entry(RegisterEntry::DB_DEPTH_CONTROL)]
    control: DB_DEPTH_CONTROL,

    #[entry(RegisterEntry::DB_DEPTH_CLEAR)]
    clear: Option<u32>,

    #[entry(RegisterEntry::DB_DEPTH_SIZE)]
    size: Option<DB_DEPTH_SIZE>,

    #[entry(RegisterEntry::DB_DEPTH_SLICE)]
    slice: Option<DB_DEPTH_SLICE>,

    #[entry(RegisterEntry::DB_DEPTH_INFO)]
    info: Option<DB_DEPTH_INFO>,

    #[entry(RegisterEntry::DB_DEPTH_VIEW)]
    view: Option<DB_DEPTH_VIEW>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct Z {
    #[entry(RegisterEntry::DB_Z_READ_BASE)]
    read_base: Option<u32>,

    #[entry(RegisterEntry::DB_Z_WRITE_BASE)]
    write_base: Option<u32>,

    #[entry(RegisterEntry::DB_Z_INFO)]
    info: Option<DB_Z_INFO>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct Stencil {
    #[entry(RegisterEntry::DB_STENCIL_READ_BASE)]
    read_base: Option<u32>,

    #[entry(RegisterEntry::DB_STENCIL_WRITE_BASE)]
    write_base: Option<u32>,

    #[entry(RegisterEntry::DB_STENCIL_INFO)]
    info: Option<DB_STENCIL_INFO>,
}

// todo: think about color1
#[derive(Build, Debug)]
#[entry(RegisterEntry)]
pub struct ColorBuffer {
    #[entry(RegisterEntry::CB_COLOR0_BASE)]
    pub base: u32,
    #[entry(RegisterEntry::CB_COLOR0_PITCH)]
    pitch: CB_COLOR0_PITCH,
    #[entry(RegisterEntry::CB_COLOR0_SLICE)]
    slice: CB_COLOR0_SLICE,
    #[entry(RegisterEntry::CB_COLOR0_VIEW)]
    view: CB_COLOR0_VIEW,
    #[entry(RegisterEntry::CB_COLOR0_INFO)]
    info: CB_COLOR0_INFO,
    #[entry(RegisterEntry::CB_COLOR0_ATTRIB)]
    attrib: CB_COLOR0_ATTRIB,
    #[entry(RegisterEntry::CB_COLOR0_CMASK)]
    mask: u32,
    #[entry(RegisterEntry::CB_COLOR0_CMASK_SLICE)]
    mask_slice: CB_COLOR0_CMASK_SLICE,
    #[entry(RegisterEntry::CB_COLOR0_FMASK)]
    fmask: u32,
    #[entry(RegisterEntry::CB_COLOR0_FMASK_SLICE)]
    fmask_slice: CB_COLOR0_SLICE,
    #[entry(RegisterEntry::CB_COLOR0_CLEAR_WORD0)]
    clear_word_0: u32,
    #[entry(RegisterEntry::CB_COLOR0_CLEAR_WORD1)]
    clear_word_1: u32,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct PrimitiveAssembly {
    clip: Clip,
    scissor_clip: ScissorClip,
    shader_unit: ShaderUnit,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct Clip {
    viewport: ClipViewport,

    #[entry(RegisterEntry::PA_CL_VTE_CNTL)]
    viewport_transform_engine_control: PA_CL_VTE_CNTL,

    #[entry(RegisterEntry::PA_CL_VS_OUT_CNTL)]
    vertex_shader_out_control: Option<PA_CL_VS_OUT_CNTL>,

    guard_band: GuardBand,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct ClipViewport {
    #[entry(RegisterEntry::PA_CL_VPORT_XSCALE)]
    xscale: u32,

    #[entry(RegisterEntry::PA_CL_VPORT_XOFFSET)]
    xoffset: u32,

    #[entry(RegisterEntry::PA_CL_VPORT_YSCALE)]
    yscale: u32,

    #[entry(RegisterEntry::PA_CL_VPORT_YOFFSET)]
    yoffset: u32,

    #[entry(RegisterEntry::PA_CL_VPORT_ZSCALE)]
    zscale: u32,

    #[entry(RegisterEntry::PA_CL_VPORT_ZOFFSET)]
    zoffset: u32,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct ScissorClip {
    #[entry(RegisterEntry::PA_SC_VPORT_ZMIN_0)]
    viewport_zmin0: u32,

    #[entry(RegisterEntry::PA_SC_VPORT_ZMAX_0)]
    viewport_zmax0: u32,

    #[entry(RegisterEntry::PA_SC_SCREEN_SCISSOR_TL)]
    screen_scissor_top_left: PA_SC_SCREEN_SCISSOR_TL,

    #[entry(RegisterEntry::PA_SC_SCREEN_SCISSOR_BR)]
    screen_scissor_bottom_right: PA_SC_SCREEN_SCISSOR_BR,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct ShaderUnit {
    #[entry(RegisterEntry::PA_SU_HARDWARE_SCREEN_OFFSET)]
    hardware_screen_offset: PA_SU_HARDWARE_SCREEN_OFFSET,

    #[entry(RegisterEntry::PA_SU_SC_MODE_CNTL)]
    control: Option<PA_SU_SC_MODE_CNTL>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct GuardBand {
    #[entry(RegisterEntry::PA_CL_GB_VERT_CLIP_ADJ)]
    vertical_clip: u32,

    #[entry(RegisterEntry::PA_CL_GB_VERT_DISC_ADJ)]
    vertical_discard: u32,

    #[entry(RegisterEntry::PA_CL_GB_HORZ_CLIP_ADJ)]
    horizontal_clip: u32,

    #[entry(RegisterEntry::PA_CL_GB_HORZ_DISC_ADJ)]
    horizontal_discard: u32,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
struct Shader {
    #[entry(RegisterEntry::SPI_SHADER_Z_FORMAT)]
    z_format: SPI_SHADER_Z_FORMAT,

    #[entry(RegisterEntry::SPI_SHADER_COL_FORMAT)]
    col_format: SPI_SHADER_COL_FORMAT,

    #[entry(RegisterEntry::SPI_SHADER_POS_FORMAT)]
    pos_format: Option<SPI_SHADER_POS_FORMAT>,

    #[entry(RegisterEntry::SPI_BARYC_CNTL)]
    barycentric_control: SPI_BARYC_CNTL,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
pub struct PixelShader {
    #[entry(RegisterEntry::SPI_SHADER_PGM_LO_PS)]
    pub address: u32,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC1_PS)]
    resource1: SPI_SHADER_PGM_RSRC1_PS,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC2_PS)]
    resource2: SPI_SHADER_PGM_RSRC2_PS,

    #[entry(RegisterEntry::SPI_PS_INPUT_ENA)]
    input: SPI_PS_INPUT_ENA,

    #[entry(RegisterEntry::SPI_PS_INPUT_ADDR)]
    input_address: SPI_PS_INPUT_ENA,

    #[entry(RegisterEntry::SPI_PS_IN_CONTROL)]
    in_control: SPI_PS_IN_CONTROL,

    #[entry(RegisterEntry::SPI_PS_INPUT_CNTL_0)]
    input_control: Option<SPI_PS_INPUT_CNTL_0>,

    pub user_data: PixelShaderUserData,
}

#[derive(Debug, BuildUserData)]
#[user_data(SPI_SHADER_USER_DATA_PS_, 0..=15)]
pub struct PixelShaderUserData(pub Vec<UserDataEntry>);

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
pub struct VertexShader {
    #[entry(RegisterEntry::SPI_SHADER_PGM_LO_VS)]
    pub address: u32,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC1_VS)]
    resource1: SPI_SHADER_PGM_RSRC1_VS,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC2_VS)]
    resource2: SPI_SHADER_PGM_RSRC2_VS,

    #[entry(RegisterEntry::SPI_VS_OUT_CONFIG)]
    out_config: SPI_VS_OUT_CONFIG,

    pub user_data: VertexShaderUserData,
}

#[derive(Debug, BuildUserData)]
#[user_data(SPI_SHADER_USER_DATA_VS_, 0..=15)]
pub struct VertexShaderUserData(pub Vec<UserDataEntry>);

// todo: crash on duplicate value

// // todo:
// // todo: positional nop entries
// // how do we handle the positional stuff?
// // Ignore it i think for now but in the future operate on the array?

#[derive(Debug, Clone)]
pub struct UserDataEntry {
    pub slot: u8,
    pub value: u32,
}
