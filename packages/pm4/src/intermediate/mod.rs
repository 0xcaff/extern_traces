pub mod build;

use crate::dispatch_direct::DispatchDirectPacket;
use crate::draw_index_auto::DrawIndexAutoPacket;
use crate::event_write_end_of_pipe::EventWriteEndOfPipePacket;
use crate::event_write_end_of_shader::EventWriteEndOfShaderPacket;
use crate::intermediate::build::{Build, Builder, Finalize, Initialize};
use crate::op_codes::OpCode;
use crate::register::{
    SetContextRegisterPacket, SetShaderRegisterPacket, SetUConfigRegisterPacket,
};
use crate::{
    PM4Packet, RegisterEntry, ShaderType, Type3Header, Type3Packet, Type3PacketValue,
    CB_COLOR0_ATTRIB, CB_COLOR0_CMASK_SLICE, CB_COLOR0_INFO, CB_COLOR0_PITCH, CB_COLOR0_SLICE,
    CB_COLOR0_VIEW, CB_SHADER_MASK, CB_TARGET_MASK, COMPUTE_NUM_THREAD_X, COMPUTE_PGM_HI,
    COMPUTE_PGM_RSRC1, COMPUTE_PGM_RSRC2, DB_DEPTH_CONTROL, DB_DEPTH_INFO, DB_DEPTH_SIZE,
    DB_DEPTH_SLICE, DB_DEPTH_VIEW, DB_HTILE_SURFACE, DB_RENDER_CONTROL, DB_SHADER_CONTROL,
    DB_STENCIL_INFO, DB_Z_INFO, PA_CL_VS_OUT_CNTL, PA_CL_VTE_CNTL, PA_SC_SCREEN_SCISSOR_BR,
    PA_SC_SCREEN_SCISSOR_TL, PA_SU_HARDWARE_SCREEN_OFFSET, PA_SU_SC_MODE_CNTL, SPI_BARYC_CNTL,
    SPI_PS_INPUT_CNTL_0, SPI_PS_INPUT_ENA, SPI_PS_IN_CONTROL, SPI_SHADER_COL_FORMAT,
    SPI_SHADER_PGM_RSRC1_PS, SPI_SHADER_PGM_RSRC1_VS, SPI_SHADER_PGM_RSRC2_PS,
    SPI_SHADER_PGM_RSRC2_VS, SPI_SHADER_POS_FORMAT, SPI_SHADER_Z_FORMAT, SPI_VS_OUT_CONFIG,
    VGT_PRIMITIVE_TYPE,
};
use pm4_internal_macros::{Build, BuildUserData};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Command {
    Draw {
        draw_packet: DrawIndexAutoPacket,
        pipeline: GraphicsPipeline,
    },
    Dispatch {
        dispatch_packet: DispatchDirectPacket,
        pipeline: ComputePipeline,
    },
    EndOfPipe(EventWriteEndOfPipePacket),
    EndOfShader(EventWriteEndOfShaderPacket),
}

pub fn convert(
    commands: &[PM4Packet],
) -> Result<(Vec<Command>, Vec<&PM4Packet>, Vec<&RegisterEntry>), anyhow::Error> {
    let mut result = vec![];
    let mut ignored_packets = vec![];
    let mut ignored_registers = vec![];

    let mut graphics_pipeline_builder = <GraphicsPipeline as Build<RegisterEntry>>::Builder::new();
    let mut compute_pipeline_builder = <ComputePipeline as Build<RegisterEntry>>::Builder::new();

    for packet in commands {
        match packet {
            PM4Packet::Type3(Type3Packet {
                header: Type3Header { shader_type, .. },
                value: Type3PacketValue::SetContextRegister(SetContextRegisterPacket { values }),
            })
            | PM4Packet::Type3(Type3Packet {
                header: Type3Header { shader_type, .. },
                value: Type3PacketValue::SetShaderRegister(SetShaderRegisterPacket { values }),
            })
            | PM4Packet::Type3(Type3Packet {
                header: Type3Header { shader_type, .. },
                value: Type3PacketValue::SetUConfigRegister(SetUConfigRegisterPacket { values }),
            }) => {
                for entry in values {
                    let Some(entry) = entry else {
                        continue;
                    };

                    let accepted = match shader_type {
                        ShaderType::Graphics => graphics_pipeline_builder.update(entry),
                        ShaderType::Compute => compute_pipeline_builder.update(entry),
                    };
                    if let None = accepted {
                        ignored_registers.push(entry);
                    }
                }
            }
            PM4Packet::Type3(Type3Packet {
                header:
                    Type3Header {
                        shader_type: ShaderType::Graphics,
                        ..
                    },
                value: Type3PacketValue::DrawIndexAuto(draw_packet),
            }) => {
                result.push(Command::Draw {
                    draw_packet: draw_packet.clone(),
                    pipeline: graphics_pipeline_builder.clone().finalize()?,
                });
            }
            PM4Packet::Type3(Type3Packet {
                header:
                    Type3Header {
                        shader_type: ShaderType::Compute,
                        ..
                    },
                value: Type3PacketValue::DispatchDirect(dispatch_packet),
            }) => {
                result.push(Command::Dispatch {
                    dispatch_packet: dispatch_packet.clone(),
                    pipeline: compute_pipeline_builder.clone().finalize()?,
                });
            }
            PM4Packet::Type3(Type3Packet {
                header: Type3Header { shader_type, .. },
                value: Type3PacketValue::ClearState(..),
            }) => match shader_type {
                ShaderType::Graphics => {
                    graphics_pipeline_builder = GraphicsPipelineBuilder::new();
                }
                ShaderType::Compute => {
                    compute_pipeline_builder = ComputePipelineBuilder::new();
                }
            },
            PM4Packet::Type3(Type3Packet {
                header: Type3Header { shader_type, .. },
                value: Type3PacketValue::EventWriteEndOfShader(packet),
            }) => {
                match shader_type {
                    ShaderType::Graphics => {
                        graphics_pipeline_builder = GraphicsPipelineBuilder::new();
                    }
                    ShaderType::Compute => {
                        compute_pipeline_builder = ComputePipelineBuilder::new();
                    }
                }
                result.push(Command::EndOfShader(packet.clone()));
            }
            PM4Packet::Type3(Type3Packet {
                header:
                    Type3Header {
                        shader_type: ShaderType::Graphics,
                        ..
                    },
                value: Type3PacketValue::EventWriteEndOfPipe(end_of_pipe),
            }) => {
                graphics_pipeline_builder = GraphicsPipelineBuilder::new();
                result.push(Command::EndOfPipe(end_of_pipe.clone()))
            }
            PM4Packet::Type3(Type3Packet {
                value:
                    Type3PacketValue::Unknown {
                        op: OpCode::NOP, ..
                    },
                ..
            }) => {
                // skip nop packets
            }
            packet => ignored_packets.push(packet),
        }
    }

    Ok((result, ignored_packets, ignored_registers))
}

/// A structured intermediate representation of data in pm4 graphics pipeline.
///
/// The goal with this is:
/// * Validate structural assumptions about the input stream like the pipeline
///   will always include an DB_DEPT_CONTROL entry or an SPI_SHADER_PGM_LO_PS
///   will always be accompanied by a SPI_SHADER_PGM_RSRC1_PS entry.
///
/// * Express as complete a set of possible information available to downstream
///   code to get a sense of implementation completeness. Things are grouped
///   according to their names (things with the same prefixes live near each
///   other).
///
/// * Provide a representation optimized for later stages to read.
#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
pub struct GraphicsPipeline {
    pub depth_buffer: DepthBuffer,

    primitive_assembly: PrimitiveAssembly,

    pub color_buffer: ColorBuffer,

    pub vertex_grouper_tesselator: VertexGrouperTesselator,

    shader: Shader,
    pub pixel_shader: PixelShader,
    pub vertex_shader: VertexShader,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
pub struct VertexGrouperTesselator {
    #[entry(RegisterEntry::VGT_PRIMITIVE_TYPE)]
    pub primitive_type: VGT_PRIMITIVE_TYPE,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
pub struct DepthBuffer {
    stencil: Stencil,
    pub z: Option<Z>,

    pub depth: Depth,

    #[entry(RegisterEntry::DB_RENDER_CONTROL)]
    pub render_control: Option<DB_RENDER_CONTROL>,

    #[entry(RegisterEntry::DB_SHADER_CONTROL)]
    shader_control: DB_SHADER_CONTROL,

    htile: Option<HTile>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
struct HTile {
    #[entry(RegisterEntry::DB_HTILE_DATA_BASE)]
    hitle_data_base: u32,

    #[entry(RegisterEntry::DB_HTILE_SURFACE)]
    htile_surface: DB_HTILE_SURFACE,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
pub struct Depth {
    #[entry(RegisterEntry::DB_DEPTH_CONTROL)]
    pub control: DB_DEPTH_CONTROL,

    #[entry(RegisterEntry::DB_DEPTH_CLEAR)]
    pub clear: Option<u32>,

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
#[allow(dead_code)]
pub struct Z {
    #[entry(RegisterEntry::DB_Z_READ_BASE)]
    pub read_base: u32,

    #[entry(RegisterEntry::DB_Z_WRITE_BASE)]
    pub write_base: u32,

    #[entry(RegisterEntry::DB_Z_INFO)]
    pub info: DB_Z_INFO,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct ColorBuffer {
    pub color0: Option<ColorBufferInstance>,

    #[entry(RegisterEntry::CB_TARGET_MASK)]
    pub target_mask: CB_TARGET_MASK,

    #[entry(RegisterEntry::CB_SHADER_MASK)]
    pub shader_mask: CB_SHADER_MASK,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
pub struct ColorBufferInstance {
    #[entry(RegisterEntry::CB_COLOR0_BASE)]
    pub base: u32,
    #[entry(RegisterEntry::CB_COLOR0_PITCH)]
    pub pitch: CB_COLOR0_PITCH,
    #[entry(RegisterEntry::CB_COLOR0_SLICE)]
    pub slice: CB_COLOR0_SLICE,
    #[entry(RegisterEntry::CB_COLOR0_VIEW)]
    pub view: CB_COLOR0_VIEW,
    #[entry(RegisterEntry::CB_COLOR0_INFO)]
    pub info: CB_COLOR0_INFO,
    #[entry(RegisterEntry::CB_COLOR0_ATTRIB)]
    pub attrib: CB_COLOR0_ATTRIB,
    #[entry(RegisterEntry::CB_COLOR0_CMASK)]
    pub mask: u32,
    #[entry(RegisterEntry::CB_COLOR0_CMASK_SLICE)]
    pub mask_slice: CB_COLOR0_CMASK_SLICE,
    #[entry(RegisterEntry::CB_COLOR0_FMASK)]
    pub fmask: u32,
    #[entry(RegisterEntry::CB_COLOR0_FMASK_SLICE)]
    pub fmask_slice: CB_COLOR0_SLICE,
    #[entry(RegisterEntry::CB_COLOR0_CLEAR_WORD0)]
    pub clear_word_0: Option<u32>,
    #[entry(RegisterEntry::CB_COLOR0_CLEAR_WORD1)]
    pub clear_word_1: Option<u32>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
struct PrimitiveAssembly {
    clip: Clip,
    scissor_clip: ScissorClip,
    shader_unit: ShaderUnit,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
struct Clip {
    viewport: ClipViewport,

    #[entry(RegisterEntry::PA_CL_VTE_CNTL)]
    viewport_transform_engine_control: PA_CL_VTE_CNTL,

    #[entry(RegisterEntry::PA_CL_VS_OUT_CNTL)]
    vertex_shader_out_control: PA_CL_VS_OUT_CNTL,

    guard_band: GuardBand,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
struct ShaderUnit {
    #[entry(RegisterEntry::PA_SU_HARDWARE_SCREEN_OFFSET)]
    hardware_screen_offset: PA_SU_HARDWARE_SCREEN_OFFSET,

    #[entry(RegisterEntry::PA_SU_SC_MODE_CNTL)]
    control: Option<PA_SU_SC_MODE_CNTL>,
}

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct PixelShader {
    #[entry(RegisterEntry::SPI_SHADER_PGM_LO_PS)]
    pub address: u32,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC1_PS)]
    resource1: SPI_SHADER_PGM_RSRC1_PS,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC2_PS)]
    resource2: SPI_SHADER_PGM_RSRC2_PS,

    #[entry(RegisterEntry::SPI_PS_INPUT_ENA)]
    input: Option<SPI_PS_INPUT_ENA>,

    #[entry(RegisterEntry::SPI_PS_INPUT_ADDR)]
    input_address: Option<SPI_PS_INPUT_ENA>,

    #[entry(RegisterEntry::SPI_PS_IN_CONTROL)]
    in_control: Option<SPI_PS_IN_CONTROL>,

    #[entry(RegisterEntry::SPI_PS_INPUT_CNTL_0)]
    input_control: Option<SPI_PS_INPUT_CNTL_0>,

    pub user_data: PixelShaderUserData,
}

#[derive(Debug, BuildUserData)]
#[user_data(SPI_SHADER_USER_DATA_PS_, 0..=15)]
#[allow(dead_code)]
pub struct PixelShaderUserData(pub BTreeMap<u8, u32>);

#[derive(Build, Debug)]
#[entry(RegisterEntry)]
#[allow(dead_code)]
pub struct VertexShader {
    #[entry(RegisterEntry::SPI_SHADER_PGM_LO_VS)]
    pub entrypoint_gpu_address: u32,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC1_VS)]
    resource1: SPI_SHADER_PGM_RSRC1_VS,

    #[entry(RegisterEntry::SPI_SHADER_PGM_RSRC2_VS)]
    resource2: SPI_SHADER_PGM_RSRC2_VS,

    #[entry(RegisterEntry::SPI_VS_OUT_CONFIG)]
    out_config: Option<SPI_VS_OUT_CONFIG>,

    pub user_data: VertexShaderUserData,
}

#[derive(Debug, BuildUserData)]
#[user_data(SPI_SHADER_USER_DATA_VS_, 0..=15)]
pub struct VertexShaderUserData(pub BTreeMap<u8, u32>);

// todo: crash on duplicate value

// // todo:
// // todo: positional nop entries
// // how do we handle the positional stuff?
// // Ignore it i think for now but in the future operate on the array?

#[derive(Debug, Build)]
#[entry(RegisterEntry)]
pub struct ComputePipeline {
    #[entry(RegisterEntry::COMPUTE_PGM_LO)]
    pub address_lo: u32,

    #[entry(RegisterEntry::COMPUTE_PGM_HI)]
    address_hi: COMPUTE_PGM_HI,

    #[entry(RegisterEntry::COMPUTE_PGM_RSRC1)]
    pub resource1: COMPUTE_PGM_RSRC1,

    #[entry(RegisterEntry::COMPUTE_PGM_RSRC2)]
    pub resource2: COMPUTE_PGM_RSRC2,

    #[entry(RegisterEntry::COMPUTE_NUM_THREAD_X)]
    pub thread_x: COMPUTE_NUM_THREAD_X,

    #[entry(RegisterEntry::COMPUTE_NUM_THREAD_Y)]
    pub thread_y: COMPUTE_NUM_THREAD_X,

    #[entry(RegisterEntry::COMPUTE_NUM_THREAD_Z)]
    pub thread_z: COMPUTE_NUM_THREAD_X,

    pub user_data: ComputeUserData,
}

#[derive(Debug, BuildUserData)]
#[user_data(COMPUTE_USER_DATA_, 0..=15)]
pub struct ComputeUserData(pub BTreeMap<u8, u32>);
