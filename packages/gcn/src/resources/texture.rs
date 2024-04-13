use bits::{Bits, FromBits};
use bits_macros::FromBits;
use strum::FromRepr;

#[derive(FromBits, Debug)]
#[bits(256)]
pub struct TextureResource {
    #[bits(37, 0)]
    base_addr_256: u64,

    #[bits(39, 38)]
    mtype_l2: u64,

    #[bits(51, 40)]
    min_lod: u64,

    #[bits(57, 52)]
    dfmt: u64,

    #[bits(61, 58)]
    nfmt: u64,

    #[bits(63, 62)]
    mtype: u64,

    #[bits(77, 64)]
    width: u64,

    #[bits(91, 78)]
    height: u64,

    #[bits(94, 92)]
    perf_mod: u64,

    #[bits(95, 95)]
    interlaced: bool,

    #[bits(98, 96)]
    dst_sel_x: DestinationChannelSelect,

    #[bits(101, 99)]
    dst_sel_y: DestinationChannelSelect,

    #[bits(104, 102)]
    dst_sel_z: DestinationChannelSelect,

    #[bits(107, 105)]
    dst_sel_w: DestinationChannelSelect,

    #[bits(111, 108)]
    base_level: u64,

    #[bits(115, 112)]
    last_level: u64,

    #[bits(120, 116)]
    tiling_idx: u64,

    #[bits(121, 121)]
    pow2pad: bool,

    #[bits(122, 122)]
    mtype_2: bool,

    #[bits(127, 124)]
    typ: u64,

    #[bits(140, 128)]
    depth: u64,

    #[bits(154, 141)]
    pitch: u64,

    #[bits(172, 160)]
    base_array: u64,

    #[bits(185, 173)]
    last_array: u64,

    #[bits(203, 192)]
    min_lod_warn: u64,

    #[bits(211, 204)]
    counter_bank_id: u64,

    #[bits(212, 212)]
    lod_hdw_cnt_en: bool,
}

#[derive(FromRepr, Debug)]
#[repr(u8)]
pub enum DestinationChannelSelect {
    Zero = 0,
    One = 1,
    R = 4,
    G = 5,
    B = 6,
    A = 7,
}

impl FromBits<3> for DestinationChannelSelect {
    fn from_bits(value: impl Bits) -> Self {
        DestinationChannelSelect::from_repr(value.full() as u8).unwrap()
    }
}
