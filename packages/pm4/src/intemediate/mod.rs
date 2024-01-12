use crate::{CB_COLOR0_ATTRIB, CB_COLOR0_CMASK_SLICE, CB_COLOR0_INFO, CB_COLOR0_PITCH, CB_COLOR0_SLICE, CB_COLOR0_VIEW, CB_SHADER_MASK, CB_TARGET_MASK, DB_DEPTH_CONTROL, DB_RENDER_CONTROL, DB_SHADER_CONTROL, PA_CL_VTE_CNTL, PA_SC_SCREEN_SCISSOR_BR, PA_SC_SCREEN_SCISSOR_TL, PA_SU_HARDWARE_SCREEN_OFFSET, SPI_BARYC_CNTL, SPI_PS_IN_CONTROL, SPI_PS_INPUT_ENA, SPI_SHADER_COL_FORMAT, SPI_SHADER_PGM_RSRC1_PS, SPI_SHADER_PGM_RSRC2_PS, SPI_SHADER_Z_FORMAT};

/// A structured intermediate representation of data in pm4 graphics pipeline.
///
/// The goal with this is:
/// * Validate structural assumptions about the input stream like the pipeline
///   will always include an DB_DEPT_CONTROL entry or a SPI_SHADER_PGM_LO_PS
///   will always be accompanied with a SPI_SHADER_PGM_RSRC1_PS entry.
///
/// * Express as complete a set of possible information available to downstream
///   code to get a sense of implementation completeness
///
/// * Provide a representation optimized for later stages to read.
///
#[derive(FromRegisterEntries)]
pub struct GraphicsPipeline {
    #[entry(RegisterEntry::DB_RENDER_CONTROL)]
    render_control: DB_RENDER_CONTROL,

    #[entry(RegisterEntry::DB_DEPTH_CONTROL)]
    depth_control: DB_DEPTH_CONTROL,

    primitive_assembly: PrimitiveAssembly,

    #[entry(RegisterEntry::CB_TARGET_MASK)]
    target_mask: CB_TARGET_MASK,

    shader: Shader,
    pixel_shader: PixelShader,
}

// todo: think about color1
struct ColorBuffer {
    #[entry(RegisterEntry::CB_COLOR0_BASE)]
    base: u32,
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

struct PrimitiveAssembly {
    clip: Clip,
    scissor_clip: ScissorClip,
    shader_unit: ShaderUnit,
}

struct Clip {
    viewport: ClipViewport,

    #[entry(RegisterEntry::PA_CL_VTE_CNTL)]
    viewport_transform_engine_control: PA_CL_VTE_CNTL,

    guard_band: GuardBand,
}

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

struct ShaderUnit {
    #[entry(RegisterEntry::PA_SU_HARDWARE_SCREEN_OFFSET)]
    hardware_screen_offset: PA_SU_HARDWARE_SCREEN_OFFSET,
}

struct GuardBand {
    #[entry(RegisterEntry::PA_CL_GB_VERT_CLIP_ADJ)]
    vertical_clip: u32,

    #[entry(RegisterEntry::PA_CL_GB_VERT_DISC_ADJ)]
    vertical_discard: u32,

    #[entry(RegisterEntry::PA_CL_GB_HORZ_CLIP_ADJ)]
    horizontal_clip: u32,

    #[entry(RegisterEntry::PA_CL_GB_VERT_DISC_ADJ)]
    horizontal_discard: u32,
}

struct Shader {
    #[entry(RegisterEntry::SPI_SHADER_Z_FORMAT)]
    z_format: SPI_SHADER_Z_FORMAT,

    #[entry(RegisterEntry::SPI_SHADER_COL_FORMAT)]
    col_format: SPI_SHADER_COL_FORMAT,

    #[entry(RegisterEntry::SPI_BARYC_CNTL)]
    baryc_control: SPI_BARYC_CNTL,
}

struct PixelShader {
    #[entry(RegisterEntry::SPI_SHADER_PGM_LO_PS)]
    address: u32,

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


    #[entry(RegisterEntry::SPI_SHADER)]
    db_shader_control: DB_SHADER_CONTROL,

    #[entry(RegisterEntry::CB_SHADER_MASK)]
    cb_shader_mask: CB_SHADER_MASK,

    // todo:
    // SPI_SHADER_USER_DATA_PS_0 - SPI_SHADER_USER_DATA_PS_15
    user_data: Vec<UserDataEntry>,
}

// todo:
// features: Option<>
// Vec<>
// how do we handle the positional stuff?
// Ignore it i think for now but in the future operate on the array?

struct UserDataEntry {
    slot: u8,
    value: u32,
}

