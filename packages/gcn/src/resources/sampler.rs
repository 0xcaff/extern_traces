use bits_macros::FromBits;

#[derive(FromBits, Debug, Hash, PartialEq, Eq, Clone)]
#[bits(128)]
pub struct SamplerResource {
    #[bits(2, 0)]
    pub clamp_x: u64,

    #[bits(5, 3)]
    pub clamp_y: u64,

    #[bits(8, 6)]
    pub clamp_z: u64,

    #[bits(11, 9)]
    pub max_aniso_ratio: u64,

    #[bits(14, 12)]
    pub depth_compare_func: u64,

    #[bits(15, 15)]
    pub force_unorm_coords: bool,

    #[bits(18, 16)]
    pub aniso_threshold: u64,

    #[bits(19, 19)]
    pub mc_coord_trunc: bool,

    #[bits(20, 20)]
    pub force_degamma: bool,

    #[bits(26, 21)]
    pub aniso_bias: u64,

    #[bits(27, 27)]
    pub trunc_coord: bool,

    #[bits(28, 28)]
    pub disable_cube_wrap: bool,

    #[bits(30, 29)]
    pub filter_mode: u64,

    #[bits(43, 32)]
    pub min_lod: LodFixed,

    #[bits(55, 44)]
    pub max_lod: LodFixed,

    #[bits(59, 56)]
    pub perf_mip: u64,

    #[bits(63, 60)]
    pub perf_z: u64,

    #[bits(77, 64)]
    pub lod_bias: u64,

    #[bits(83, 78)]
    pub lod_bias_sec: u64,

    #[bits(85, 84)]
    pub xy_mag_filter: u64,

    #[bits(87, 86)]
    pub xy_min_filter: u64,

    #[bits(89, 88)]
    pub z_filter: u64,

    #[bits(91, 90)]
    pub mip_filter: u64,

    #[bits(108, 107)]
    pub border_color_ptr: u64,

    #[bits(127, 126)]
    pub border_color_type: u64,
}

#[derive(FromBits, Debug, Hash, PartialEq, Eq, Clone)]
#[bits(12)]
struct LodFixed {
    #[bits(3, 0)]
    int: u64,

    #[bits(11, 4)]
    frac: u8,
}
