use bits_macros::FromBits;

#[derive(FromBits, Debug)]
#[bits(128)]
pub struct SamplerResource {
    #[bits(2, 0)]
    clamp_x: u64,

    #[bits(5, 3)]
    clamp_y: u64,

    #[bits(8, 6)]
    clamp_z: u64,

    #[bits(11, 9)]
    max_aniso_ratio: u64,

    #[bits(14, 12)]
    depth_compare_func: u64,

    #[bits(15, 15)]
    force_unorm_coords: bool,

    #[bits(18, 16)]
    aniso_threshold: u64,

    #[bits(19, 19)]
    mc_coord_trunc: bool,

    #[bits(20, 20)]
    force_degamma: bool,

    #[bits(26, 21)]
    aniso_bias: u64,

    #[bits(27, 27)]
    trunc_coord: bool,

    #[bits(28, 28)]
    disable_cube_wrap: bool,

    #[bits(30, 29)]
    filter_mode: u64,

    #[bits(43, 32)]
    min_lod: LodFixed,

    #[bits(55, 44)]
    max_lod: LodFixed,

    #[bits(59, 56)]
    perf_mip: u64,

    #[bits(63, 60)]
    perf_z: u64,

    #[bits(77, 64)]
    lod_bias: u64,

    #[bits(83, 78)]
    lod_bias_sec: u64,

    #[bits(85, 84)]
    xy_mag_filter: u64,

    #[bits(87, 86)]
    xy_min_filter: u64,

    #[bits(89, 88)]
    z_filter: u64,

    #[bits(91, 90)]
    mip_filter: u64,

    #[bits(108, 107)]
    border_color_ptr: u64,

    #[bits(127, 126)]
    border_color_type: u64,
}

#[derive(FromBits, Debug)]
#[bits(12)]
struct LodFixed {
    #[bits(3, 0)]
    int: u64,

    #[bits(11, 4)]
    frac: u8,
}
