use crate::resources::{BufferChannelType, DestinationChannelSelect};
use bits_macros::FromBits;
use core::slice;

#[derive(FromBits, Debug, Hash, Eq, PartialEq, Clone)]
#[bits(128)]
pub struct VertexBufferResource {
    #[bits(43, 0)]
    pub base: u64,

    #[bits(45, 44)]
    pub mtype_l1s: u64,

    #[bits(47, 46)]
    pub mtype_l2: u64,

    // todo: use tighter storage sizes
    #[bits(61, 48)]
    pub stride: u64,

    #[bits(62, 62)]
    pub cache_swizzle: bool,

    #[bits(63, 63)]
    pub swizzle_en: bool,

    #[bits(95, 64)]
    pub num_records: u32,

    #[bits(98, 96)]
    pub dst_sel_x: DestinationChannelSelect,

    #[bits(101, 99)]
    pub dst_sel_y: DestinationChannelSelect,

    #[bits(104, 102)]
    pub dst_sel_z: DestinationChannelSelect,

    #[bits(107, 105)]
    pub dst_sel_w: DestinationChannelSelect,

    #[bits(110, 108)]
    pub nfmt: BufferChannelType,

    #[bits(114, 111)]
    pub dfmt: u64,

    #[bits(116, 115)]
    pub element_size: u64,

    #[bits(118, 117)]
    pub index_stride: u64,

    #[bits(119, 119)]
    pub addtid_en: bool,

    #[bits(121, 121)]
    pub hash_en: bool,

    #[bits(125, 123)]
    pub mtype: u64,

    #[bits(127, 126)]
    pub typ: u64,
}

impl VertexBufferResource {
    pub fn len(&self) -> u64 {
        self.stride * self.num_records as u64
    }

    pub unsafe fn bytes<'a>(&self) -> &'a [u8] {
        slice::from_raw_parts(self.base as *const u8, self.len() as _)
    }
}
