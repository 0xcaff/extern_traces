use bits::{Bits, FromBits, TryFromBits};
use bits_macros::TryFromBitsContainer;
use strum::{EnumIter, FromRepr};

#[derive(TryFromBitsContainer, Debug, Hash, Eq, PartialEq, Clone)]
#[bits(256)]
pub struct TextureBufferResource {
    #[bits(37, 0)]
    pub base_addr_256: u64,

    #[bits(39, 38)]
    pub mtype_l2: u64,

    #[bits(51, 40)]
    pub min_lod: u64,

    #[bits(57, 52)]
    pub dfmt: SurfaceFormat,

    #[bits(61, 58)]
    pub nfmt: TextureChannelType,

    #[bits(63, 62)]
    pub mtype: u64,

    #[bits(77, 64)]
    pub width: u64,

    #[bits(91, 78)]
    pub height: u64,

    #[bits(94, 92)]
    pub perf_mod: u64,

    #[bits(95, 95)]
    pub interlaced: bool,

    #[bits(98, 96)]
    pub dst_sel_x: DestinationChannelSelect,

    #[bits(101, 99)]
    pub dst_sel_y: DestinationChannelSelect,

    #[bits(104, 102)]
    pub dst_sel_z: DestinationChannelSelect,

    #[bits(107, 105)]
    pub dst_sel_w: DestinationChannelSelect,

    #[bits(111, 108)]
    pub base_level: u64,

    #[bits(115, 112)]
    pub last_level: u64,

    #[bits(120, 116)]
    pub tiling_idx: TileMode,

    #[bits(121, 121)]
    pub pow2pad: bool,

    #[bits(122, 122)]
    pub mtype_2: bool,

    #[bits(127, 124)]
    pub texture_type: TextureType,

    #[bits(140, 128)]
    pub depth: u64,

    #[bits(154, 141)]
    pub pitch: u64,

    #[bits(172, 160)]
    pub base_array: u64,

    #[bits(185, 173)]
    pub last_array: u64,

    #[bits(203, 192)]
    pub min_lod_warn: u64,

    #[bits(211, 204)]
    pub counter_bank_id: u64,

    #[bits(212, 212)]
    pub lod_hdw_cnt_en: bool,

    // From https://github.com/Inori/GPCS4/blob/1466ffc418e73a2df8408a16fc8d2f8077519cbb/GPCS4/Graphics/Gnm/GnmTexture.h#L633-L637
    #[bits(216, 216)]
    pub minimum_gpu_mode_is_neo: bool,
}

impl TextureBufferResource {
    pub fn base_address(&self) -> u64 {
        // drop the top 6 bits, shift the remaining 32 bits into a 40-bit address
        (self.base_addr_256 & u32::MAX as u64) << 8
    }

    pub fn bytes_len(&self) -> usize {
        (self.width as usize + 1)
            * (self.height as usize + 1)
            * (self.depth as usize + 1)
            * self.dfmt.bytes_len()
    }

    pub unsafe fn bytes<'a>(&self) -> &'a [u8] {
        core::slice::from_raw_parts(self.base_address() as *const u8, self.bytes_len() as _)
    }
}

// From GPCS4
// https://github.com/Inori/GPCS4/blob/8a4376bb7908f406b80d56f5d3f5ca9da51a7478/GPCS4/Graphics/Gnm/GnmConstant.h#L1385-L1433
#[allow(non_camel_case_types)]
#[derive(FromRepr, Debug, Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum SurfaceFormat {
    /// Invalid surface format.
    FormatInvalid = 0x00,
    /// One 8-bit channel. X=0xFF
    Format8 = 0x01,
    ///< One 16-bit channel. X=0xFFFF
    Format16 = 0x02,
    ///< Two 8-bit channels. X=0x00FF, Y=0xFF00
    Format8_8 = 0x03,
    ///< One 32-bit channel. X=0xFFFFFFFF
    Format32 = 0x04,
    ///< Two 16-bit channels. X=0x0000FFFF, Y=0xFFFF0000
    Format16_16 = 0x05,
    ///< One 10-bit channel (Z) and two 11-bit channels (Y,X). X=0x000007FF, Y=0x003FF800, Z=0xFFC00000 Interpreted only as floating-point by texture unit, but also as integer by rasterizer.
    Format10_11_11 = 0x06,
    ///< Two 11-bit channels (Z,Y) and one 10-bit channel (X). X=0x000003FF, Y=0x001FFC00, Z=0xFFE00000 Interpreted only as floating-point by texture unit, but also as integer by rasterizer.
    Format11_11_10 = 0x07,
    ///< Three 10-bit channels (W,Z,Y) and one 2-bit channel (X). X=0x00000003, Y=0x00000FFC, Z=0x003FF000, W=0xFFC00000 X is never negative, even when YZW are.
    Format10_10_10_2 = 0x08,
    ///< One 2-bit channel (W) and three 10-bit channels (Z,Y,X). X=0x000003FF, Y=0x000FFC00, Z=0x3FF00000, W=0xC0000000 W is never negative, even when XYZ are.
    Format2_10_10_10 = 0x09,
    ///< Four 8-bit channels. X=0x000000FF, Y=0x0000FF00, Z=0x00FF0000, W=0xFF000000
    Format8_8_8_8 = 0x0a,
    ///< Two 32-bit channels.
    Format32_32 = 0x0b,
    ///< Four 16-bit channels.
    Format16_16_16_16 = 0x0c,
    ///< Three 32-bit channels.
    Format32_32_32 = 0x0d,
    ///< Four 32-bit channels.
    Format32_32_32_32 = 0x0e,
    ///< One 5-bit channel (Z), one 6-bit channel (Y), and a second 5-bit channel (X). X=0x001F, Y=0x07E0, Z=0xF800
    Format5_6_5 = 0x10,
    ///< One 1-bit channel (W) and three 5-bit channels (Z,Y,X). X=0x001F, Y=0x03E0, Z=0x7C00, W=0x8000
    Format1_5_5_5 = 0x11,
    ///< Three 5-bit channels (W,Z,Y) and one 1-bit channel (X). X=0x0001, Y=0x003E, Z=0x07C0, W=0xF800
    Format5_5_5_1 = 0x12,
    ///< Four 4-bit channels. X=0x000F, Y=0x00F0, Z=0x0F00, W=0xF000
    Format4_4_4_4 = 0x13,
    ///< One 8-bit channel and one 24-bit channel.
    Format8_24 = 0x14,
    ///< One 24-bit channel and one 8-bit channel.
    Format24_8 = 0x15,
    ///< One 24-bit channel, one 8-bit channel, and one 32-bit channel.
    FormatX24_8_32 = 0x16,
    ///< To be documented.
    FormatGB_GR = 0x20,
    ///< To be documented.
    FormatBG_RG = 0x21,
    ///< One 5-bit channel (W) and three 9-bit channels (Z,Y,X). X=0x000001FF, Y=0x0003FE00, Z=0x07FC0000, W=0xF8000000. Interpreted only as three 9-bit denormalized mantissas, and one shared 5-bit exponent.
    Format5_9_9_9 = 0x22,
    ///< BC1 block-compressed surface.
    FormatBc1 = 0x23,
    ///< BC2 block-compressed surface.
    FormatBc2 = 0x24,
    ///< BC3 block-compressed surface.
    FormatBc3 = 0x25,
    ///< BC4 block-compressed surface.
    FormatBc4 = 0x26,
    ///< BC5 block-compressed surface.
    FormatBc5 = 0x27,
    ///< BC6 block-compressed surface.
    FormatBc6 = 0x28,
    ///< BC7 block-compressed surface.
    FormatBc7 = 0x29,
    ///< 8 bits-per-element FMASK surface (2 samples, 1 fragment).
    FormatFmask8_S2_F1 = 0x2C,
    ///< 8 bits-per-element FMASK surface (4 samples, 1 fragment).
    FormatFmask8_S4_F1 = 0x2D,
    ///< 8 bits-per-element FMASK surface (8 samples, 1 fragment).
    FormatFmask8_S8_F1 = 0x2E,
    ///< 8 bits-per-element FMASK surface (2 samples, 2 fragments).
    FormatFmask8_S2_F2 = 0x2F,
    ///< 8 bits-per-element FMASK surface (8 samples, 2 fragments).
    FormatFmask8_S4_F2 = 0x30,
    ///< 8 bits-per-element FMASK surface (4 samples, 4 fragments).
    FormatFmask8_S4_F4 = 0x31,
    ///< 16 bits-per-element FMASK surface (16 samples, 1 fragment).
    FormatFmask16_S16_F1 = 0x32,
    ///< 16 bits-per-element FMASK surface (8 samples, 2 fragments).
    FormatFmask16_S8_F2 = 0x33,
    ///< 32 bits-per-element FMASK surface (16 samples, 2 fragments).
    FormatFmask32_S16_F2 = 0x34,
    ///< 32 bits-per-element FMASK surface (8 samples, 4 fragments).
    FormatFmask32_S8_F4 = 0x35,
    ///< 32 bits-per-element FMASK surface (8 samples, 8 fragments).
    FormatFmask32_S8_F8 = 0x36,
    ///< 64 bits-per-element FMASK surface (16 samples, 4 fragments).
    FormatFmask64_S16_F4 = 0x37,
    ///< 64 bits-per-element FMASK surface (16 samples, 8 fragments).
    FormatFmask64_S16_F8 = 0x38,
    ///< Two 4-bit channels (Y,X). X=0x0F, Y=0xF0
    Format4_4 = 0x39,
    ///< One 6-bit channel (Z) and two 5-bit channels (Y,X). X=0x001F, Y=0x03E0, Z=0xFC00
    Format6_5_5 = 0x3A,
    ///< One 1-bit channel. 8 pixels per byte, with pixel index increasing from LSB to MSB.
    Format1 = 0x3B,
    ///< One 1-bit channel. 8 pixels per byte, with pixel index increasing from MSB to LSB.
    Format1Reversed = 0x3C,
}

impl SurfaceFormat {
    pub fn bytes_len(&self) -> usize {
        match self {
            SurfaceFormat::Format8 => 1,
            SurfaceFormat::Format16 => 2,
            SurfaceFormat::Format8_8 => 2,
            SurfaceFormat::Format32 => 4,
            SurfaceFormat::Format16_16 => 4,
            SurfaceFormat::Format10_11_11 => 4,
            SurfaceFormat::Format11_11_10 => 4,
            SurfaceFormat::Format10_10_10_2 => 4,
            SurfaceFormat::Format2_10_10_10 => 4,
            SurfaceFormat::Format8_8_8_8 => 4,
            SurfaceFormat::Format32_32 => 8,
            SurfaceFormat::Format16_16_16_16 => 8,
            SurfaceFormat::Format32_32_32 => 12,
            SurfaceFormat::Format32_32_32_32 => 16,
            SurfaceFormat::Format5_6_5 => 2,
            _ => unimplemented!(),
        }
    }
}

impl TryFromBits<6> for SurfaceFormat {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        Self::from_repr(value.full() as u8)
    }
}

#[derive(FromRepr, Debug, Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum BufferChannelType {
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float X/(N-1)</c>.
    UNorm = 0x00000000,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>float MAX(-1,X/(N/2-1))</c>.
    SNorm = 0x00000001,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float X</c>.
    UScaled = 0x00000002,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>float X</c>.
    SScaled = 0x00000003,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>uint X</c>. Not filterable.
    UInt = 0x00000004,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>int X</c>. Not filterable.
    SInt = 0x00000005,
    ///< Stored as <c>int -N/2<=X\<N/2</c>, interpreted as <c>float ((X+N/2)/(N-1))*2-1</c>.
    SNormNoZero = 0x00000006,
    ///< Stored as <c>float</c>, interpreted as <c>float</c>.
    Float = 0x00000007,
    //<  – 32-bit: SE8M23, bias 127, range <c>(-2^129..2^129)</c>
    //<  – 16-bit: SE5M10, bias 15, range <c>(-2^17..2^17)</c>
    //<  – 11-bit: E5M6 bias 15, range <c>[0..2^17)</c>
    //<  – 10-bit: E5M5 bias 15, range <c>[0..2^17)</c>
}

impl FromBits<3> for BufferChannelType {
    fn from_bits(value: impl Bits) -> Self {
        Self::from_repr(value.full() as u8).unwrap()
    }
}

#[derive(FromRepr, Debug, Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum TextureChannelType {
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float X/(N-1)</c>.
    UNorm = 0x00000000,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>float MAX(-1,X/(N/2-1))</c>.
    SNorm = 0x00000001,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float X</c>.
    UScaled = 0x00000002,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>float X</c>.
    SScaled = 0x00000003,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>uint X</c>. Not filterable.
    UInt = 0x00000004,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>int X</c>. Not filterable.
    SInt = 0x00000005,
    ///< Stored as <c>int -N/2\<=X\<N/2</c>, interpreted as <c>float ((X+N/2)/(N-1))*2-1</c>.
    SNormNoZero = 0x00000006,
    ///< Stored as <c>float</c>, interpreted as <c>float</c>.
    Float = 0x00000007,
    //<  – 32-bit: SE8M23, bias 127, range <c>(-2^129..2^129)</c>
    //<  – 16-bit: SE5M10, bias 15, range <c>(-2^17..2^17)</c>
    //<  – 11-bit: E5M6 bias 15, range <c>[0..2^17)</c>
    //<  – 10-bit: E5M5 bias 15, range <c>[0..2^17)</c>
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float sRGB(X/(N-1))</c>. Srgb only applies to the XYZ channels of the texture; the W channel is always linear.
    Srgb = 0x00000009,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float MAX(-1,(X-N/2)/(N/2-1))</c>.
    UBNorm = 0x0000000A,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float (X/(N-1))*2-1</c>.
    UBNormNoZero = 0x0000000B,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>int X-N/2</c>. Not blendable or filterable.
    UBInt = 0x0000000C,
    ///< Stored as <c>uint X\<N</c>, interpreted as <c>float X-N/2</c>.
    UBScaled = 0x0000000D,
}

impl TryFromBits<4> for TextureChannelType {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        Self::from_repr(value.full() as u8)
    }
}

// From https://github.com/Inori/GPCS4/blob/8a4376bb7908f406b80d56f5d3f5ca9da51a7478/GPCS4/Graphics/Gnm/GnmConstant.h#L1820-L1827
#[derive(FromRepr, Debug, Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum TextureType {
    ///< One-dimensional texture.
    Type1d = 0x00000008,
    ///< Two-dimensional texture.
    Type2d = 0x00000009,
    ///< Three-dimensional volume texture.
    Type3d = 0x0000000A,
    ///< Cubic environment map texture (six 2D textures arranged in a cube and indexed by a 3D direction vector). This TextureType is also used for cubemap arrays.
    TypeCubemap = 0x0000000B,
    ///< Array of 1D textures.
    Type1dArray = 0x0000000C,
    ///< Array of 2D textures.
    Type2dArray = 0x0000000D,
    ///< Two-dimensional texture with multiple samples per pixel. Usually an alias into an MSAA render target.
    Type2dMsaa = 0x0000000E,
    ///< Array of 2D MSAA textures.
    Type2dArrayMsaa = 0x0000000F,
}

impl TryFromBits<4> for TextureType {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        Self::from_repr(value.full() as u8)
    }
}

// From https://github.com/Inori/GPCS4/blob/8a4376bb7908f406b80d56f5d3f5ca9da51a7478/GPCS4/Graphics/Gnm/GnmConstant.h#L1542-L1574
#[derive(FromRepr, Debug, Hash, Eq, PartialEq, Clone, EnumIter, Copy)]
#[repr(u8)]
pub enum TileMode {
    // Depth modes (for depth buffers)
    ///< Recommended for depth targets with one fragment per pixel.
    Depth2dThin64 = 0x00000000,
    ///< Recommended for depth targets with two or four fragments per pixel, or texture-readable.
    Depth2dThin128 = 0x00000001,
    ///< Recommended for depth targets with eight fragments per pixel.
    Depth2dThin256 = 0x00000002,
    ///< Recommended for depth targets with 512-byte tiles.
    Depth2dThin512 = 0x00000003,
    ///< Recommended for depth targets with 1024-byte tiled.
    Depth2dThin1k = 0x00000004,
    ///< Not used; included only for completeness.
    Depth1dThin = 0x00000005,
    ///< Recommended for partially-resident depth surfaces. Does not support aliasing multiple virtual texture pages to the same physical page.
    Depth2dThinPrt256 = 0x00000006,
    ///< Not used; included only for completeness.
    Depth2dThinPrt1k = 0x00000007,
    // Display modes
    ///< Recommended for any surface to be easily accessed on the CPU.
    DisplayLinearAligned = 0x00000008,
    ///< Not used; included only for completeness.
    Display1dThin = 0x00000009,
    ///< Recommended mode for displayable render targets.
    Display2dThin = 0x0000000A,
    ///< Supports aliasing multiple virtual texture pages to the same physical page.
    DisplayThinPrt = 0x0000000B,
    ///< Does not support aliasing multiple virtual texture pages to the same physical page.
    Display2dThinPrt = 0x0000000C,
    // Thin modes (for non-displayable 1D/2D/3D surfaces)
    ///< Recommended for read-only non-volume textures.
    Thin1dThin = 0x0000000D,
    ///< Recommended for non-displayable intermediate render targets and read/write non-volume textures.
    Thin2dThin = 0x0000000E,
    ///< Not used; included only for completeness.
    Thin3dThin = 0x0000000F,
    ///< Recommended for partially-resident textures (PRTs). Supports aliasing multiple virtual texture pages to the same physical page.
    ThinThinPrt = 0x00000010,
    ///< Does not support aliasing multiple virtual texture pages to the same physical page.
    Thin2dThinPrt = 0x00000011,
    ///< Does not support aliasing multiple virtual texture pages to the same physical page.
    Thin3dThinPrt = 0x00000012,
    // Thick modes (for 3D textures)
    ///< Recommended for read-only volume textures.
    Thick1dThick = 0x00000013,
    ///< Recommended for volume textures to which pixel shaders will write.
    Thick2dThick = 0x00000014,
    ///< Not used; included only for completeness.
    Thick3dThick = 0x00000015,
    ///< Supports aliasing multiple virtual texture pages to the same physical page.
    ThickThickPrt = 0x00000016,
    ///< Does not support aliasing multiple virtual texture pages to the same physical page.
    Thick2dThickPrt = 0x00000017,
    ///< Does not support aliasing multiple virtual texture pages to the same physical page.
    Thick3dThickPrt = 0x00000018,
    ///< Recommended for volume textures to which pixel shaders will write.
    Thick2dXthick = 0x00000019,
    ///< Not used; included only for completeness.
    Thick3dXthick = 0x0000001A,
    // Hugely inefficient linear display mode - do not use!
    ///< Unsupported; do not use!
    DisplayLinearGeneral = 0x0000001F,
}

impl TryFromBits<5> for TileMode {
    fn try_from_bits(value: impl Bits) -> Option<Self> {
        Self::from_repr(value.full() as u8)
    }
}

#[derive(FromRepr, Debug, Hash, Eq, PartialEq, Clone)]
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
