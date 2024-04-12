use bits_macros::FromBits;

#[derive(FromBits)]
#[bits(128)]
struct SamplerResource {
    #[bits(2, 0)]
    clamp_x: usize,

    #[bits(5, 3)]
    clamp_y: usize,

    #[bits(8, 6)]
    clamp_z: usize,

    #[bits(11, 9)]
    max_aniso_ratio: usize,

    #[bits(14, 12)]
    depth_compare_func: usize,

    #[bits(15, 15)]
    force_unorm_coords: bool,

    #[bits(18, 16)]
    aniso_threshold: usize,

    #[bits(19, 19)]
    mc_coord_trunc: bool,

    #[bits(20, 20)]
    force_degamma: bool,

    #[bits(26, 21)]
    aniso_bias: usize,

    #[bits(27, 27)]
    trunc_coord: bool,

    #[bits(28, 28)]
    disable_cube_wrap: bool,

    #[bits(30, 29)]
    filter_mode: usize,

    #[bits(43, 32)]
    min_lod: LodFixed,

    #[bits(55, 44)]
    max_lod: LodFixed,

    #[bits(59, 56)]
    perf_mip: usize,

    #[bits(63, 60)]
    perf_z: usize,

    #[bits(77, 64)]
    lod_bias: usize,

    #[bits(83, 78)]
    lod_bias_sec: usize,

    #[bits(85, 84)]
    xy_mag_filter: usize,

    #[bits(87, 86)]
    xy_min_filter: usize,

    #[bits(89, 88)]
    z_filter: usize,

    #[bits(91, 90)]
    mip_filter: usize,

    #[bits(108, 107)]
    border_color_ptr: usize,

    #[bits(127, 126)]
    border_color_type: usize,
}

#[derive(FromBits)]
#[bits(12)]
struct LodFixed {
    #[bits(3, 0)]
    int: usize,

    #[bits(11, 4)]
    frac: u8,
}
