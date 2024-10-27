use gcn::resources::{SurfaceFormat, TextureBufferResource, TileMode};
use std::cmp::min;
use strum::{EnumIter, FromRepr};

pub fn detile(
    texture_buffer_resource: &TextureBufferResource,
    bytes: &[u32],
    out_bytes: &mut [u32],
) {
    for idx in 0..bytes.len() {
        let column = (idx as u64 % (texture_buffer_resource.pitch + 1)) as u32;
        let row = (idx as u64 / (texture_buffer_resource.pitch + 1)) as u32;
        let source_idx = zorder::index_of([column, row]) as usize;
        out_bytes[idx] = bytes[source_idx];
    }
}

// From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GnmConstant.h#L1477-L1487
#[derive(Copy, Clone, FromRepr, Ord, PartialOrd, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum TileSplit {
    k64B = 0x00000000,
    k128B = 0x00000001,
    k256B = 0x00000002,
    k512B = 0x00000003,
    k1KB = 0x00000004,
    k2KB = 0x00000005,
    k4KB = 0x00000006,
}

// From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GnmConstant.h#L1489-L1496
#[derive(Copy, Clone, FromRepr, Ord, PartialOrd, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SampleSplit {
    k1 = 0x00000000,
    k2 = 0x00000001,
    k4 = 0x00000002,
    k8 = 0x00000003,
}

// From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GnmConstant.h#L1437-L1444
#[derive(Copy, Clone, FromRepr, Ord, PartialOrd, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MicroTileMode {
    Display = 0x00000000, //< Only for 64 bpp and below.
    Thin = 0x00000001,    //< Non-displayable. Can be used for thin, thick, or X thick.
    Depth = 0x00000002,   //< Only mode supported by DB.
    Rotated = 0x00000003, //< Rotated. Not supported by Gnm.
    Thick = 0x00000004,   //< Thick and X thick, non-AA only.
}

// From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GnmConstant.h#L1449-L1467
#[derive(Copy, Clone, FromRepr, Ord, PartialOrd, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ArrayMode {
    kLinearGeneral = 0x00000000, //< Linear pixel storage; no alignment or padding restrictions. DEPRECATED -- Do not use!
    kLinearAligned = 0x00000001, //< Linear pixel storage with some minor alignment requirements and internal padding.
    k1dTiledThin = 0x00000002, //< Micro-tile-only tiling for non-volume surfaces. Not valid for AA modes.
    k1dTiledThick = 0x00000003, //< Micro-tile-only tiling for volume surfaces (8x8x4 pixel micro-tiles). Not valid for AA modes.
    k2dTiledThin = 0x00000004,  //< Macro-tile tiling for non-volume surfaces.
    kTiledThinPrt = 0x00000005, //< Macro-tile tiling for non-volume partially-resident texture (PRT) surfaces. Supports aliasing multiple virtual texture pages to the same physical page.
    k2dTiledThinPrt = 0x00000006, //< Macro-tile tiling for non-volume partially-resident texture (PRT) surfaces. Does not support aliasing multiple virtual texture pages to the same physical page.
    k2dTiledThick = 0x00000007, //< Macro-tile tiling for volume surfaces (8x8x4 pixel micro-tiles).
    k2dTiledXThick = 0x00000008, //< Macro-tile tiling for volume surfaces (8x8x8 pixel micro-tiles).
    kTiledThickPrt = 0x00000009, //< Micro-tile-only tiling for partially-resident texture (PRT) volume surfaces (8x8x4 pixel micro-tiles). Supports aliasing multiple virtual texture pages to the same physical page.
    k2dTiledThickPrt = 0x0000000a, //< Macro-tile tiling for partially-resident texture (PRT) volume surfaces (8x8x4 pixel micro-tiles). Does not support aliasing multiple virtual texture pages to the same physical page.
    k3dTiledThinPrt = 0x0000000b, //< Macro-tile tiling for partially-resident texture (PRT) non-volume surfaces. Z slices are rotated by pipe. Does not support aliasing multiple virtual texture pages to the same physical page.
    k3dTiledThin = 0x0000000c, //< Macro-tile tiling for non-volume surfaces. Z slices are rotated by pipe.
    k3dTiledThick = 0x0000000d, //< Macro-tile tiling for volume surfaces (8x8x4 pixel micro-tiles). Z slices are rotated by pipe.
    k3dTiledXThick = 0x0000000e, //< Macro-tile tiling for volume surfaces (8x8x8 pixel micro-tiles). Z slices are rotated by pipe.
    k3dTiledThickPrt = 0x0000000f, //< Macro-tile tiling for partially-resident texture (PRT) volume surfaces (8x8x4 pixel micro-tiles). Z slices are rotated by pipe. Does not support aliasing multiple virtual texture pages to the same physical page.
}

impl ArrayMode {
    // From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GpuAddress/GnmGpuAddressInternal.cpp#L98-L129
    pub fn thickness(&self) -> u32 {
        match self {
            // Thick variants (thickness = 4)
            ArrayMode::k1dTiledThick
            | ArrayMode::k2dTiledThick
            | ArrayMode::k3dTiledThick
            | ArrayMode::kTiledThickPrt
            | ArrayMode::k2dTiledThickPrt
            | ArrayMode::k3dTiledThickPrt => 4,

            // XThick variants (thickness = 8)
            ArrayMode::k2dTiledXThick | ArrayMode::k3dTiledXThick => 8,

            // All other variants (thickness = 1)
            ArrayMode::kLinearGeneral
            | ArrayMode::kLinearAligned
            | ArrayMode::k1dTiledThin
            | ArrayMode::k2dTiledThin
            | ArrayMode::kTiledThinPrt
            | ArrayMode::k2dTiledThinPrt
            | ArrayMode::k3dTiledThinPrt
            | ArrayMode::k3dTiledThin => 1,
        }
    }

    // From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GpuAddress/GnmGpuAddressInternal.cpp#L197-L218
    pub fn is_partially_resident_texture(&self) -> bool {
        match self {
            ArrayMode::kTiledThinPrt
            | ArrayMode::k2dTiledThinPrt
            | ArrayMode::kTiledThickPrt
            | ArrayMode::k2dTiledThickPrt
            | ArrayMode::k3dTiledThinPrt
            | ArrayMode::k3dTiledThickPrt => true,
            ArrayMode::kLinearGeneral
            | ArrayMode::kLinearAligned
            | ArrayMode::k1dTiledThin
            | ArrayMode::k1dTiledThick
            | ArrayMode::k2dTiledThin
            | ArrayMode::k2dTiledThick
            | ArrayMode::k2dTiledXThick
            | ArrayMode::k3dTiledThin
            | ArrayMode::k3dTiledThick
            | ArrayMode::k3dTiledXThick => false,
        }
    }
}

// From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GnmConstant.h#L1470-L1475
#[derive(Copy, Clone, FromRepr, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PipeConfig {
    P8_32x32_8x16 = 0x0000000a,
    P8_32x32_16x16 = 0x0000000c,
    P16 = 0x00000012,
}

impl PipeConfig {
    pub fn index(&self, x: u32, y: u32) -> u32 {
        let mut pipe = 0;
        match self {
            PipeConfig::P8_32x32_8x16 => {
                pipe |= (((x >> 4) ^ (y >> 3) ^ (x >> 5)) & 0x1) << 0;
                pipe |= (((x >> 3) ^ (y >> 4)) & 0x1) << 1;
                pipe |= (((x >> 5) ^ (y >> 5)) & 0x1) << 2;
            }
            PipeConfig::P8_32x32_16x16 => {
                pipe |= (((x >> 3) ^ (y >> 3) ^ (x >> 4)) & 0x1) << 0;
                pipe |= (((x >> 4) ^ (y >> 4)) & 0x1) << 1;
                pipe |= (((x >> 5) ^ (y >> 5)) & 0x1) << 2;
            }
            PipeConfig::P16 => {
                pipe |= (((x >> 3) ^ (y >> 3) ^ (x >> 4)) & 0x1) << 0;
                pipe |= (((x >> 4) ^ (y >> 4)) & 0x1) << 1;
                pipe |= (((x >> 5) ^ (y >> 5)) & 0x1) << 2;
                pipe |= (((x >> 6) ^ (y >> 5)) & 0x1) << 3;
            }
        }

        pipe
    }

    pub fn count(&self) -> u32 {
        match self {
            PipeConfig::P8_32x32_8x16 | PipeConfig::P8_32x32_16x16 => 8,
            PipeConfig::P16 => 16,
        }
    }
}

trait TileModeExt {
    fn tile_split(&self) -> TileSplit;
    fn sample_split(&self) -> SampleSplit;
    fn micro_tile_mode(&self) -> MicroTileMode;
    fn array_mode(&self) -> ArrayMode;
    fn pipe_config(&self) -> Option<PipeConfig>;
}

impl TileModeExt for TileMode {
    fn tile_split(&self) -> TileSplit {
        match self {
            TileMode::Depth2dThin128 => TileSplit::k128B,
            TileMode::Depth2dThin256 | TileMode::Depth2dThinPrt256 => TileSplit::k256B,
            TileMode::Depth2dThin512 => TileSplit::k512B,
            TileMode::Depth2dThin1k | TileMode::Depth2dThinPrt1k => TileSplit::k1KB,
            _ => TileSplit::k64B,
        }
    }

    fn sample_split(&self) -> SampleSplit {
        match self {
            TileMode::Display2dThin
            | TileMode::DisplayThinPrt
            | TileMode::Display2dThinPrt
            | TileMode::Thin2dThin
            | TileMode::Thin3dThin
            | TileMode::ThinThinPrt
            | TileMode::Thin2dThinPrt
            | TileMode::Thin3dThinPrt => SampleSplit::k2,
            _ => SampleSplit::k1,
        }
    }

    fn micro_tile_mode(&self) -> MicroTileMode {
        match self {
            TileMode::Depth2dThin64
            | TileMode::Depth2dThin128
            | TileMode::Depth2dThin256
            | TileMode::Depth2dThin512
            | TileMode::Depth2dThin1k
            | TileMode::Depth1dThin
            | TileMode::Depth2dThinPrt256
            | TileMode::Depth2dThinPrt1k => MicroTileMode::Depth,

            TileMode::Thin1dThin
            | TileMode::Thin2dThin
            | TileMode::Thin3dThin
            | TileMode::ThinThinPrt
            | TileMode::Thin2dThinPrt
            | TileMode::Thin3dThinPrt => MicroTileMode::Thin,

            TileMode::Thick1dThick
            | TileMode::Thick2dThick
            | TileMode::Thick3dThick
            | TileMode::ThickThickPrt
            | TileMode::Thick2dThickPrt
            | TileMode::Thick3dThickPrt
            | TileMode::Thick2dXthick
            | TileMode::Thick3dXthick => MicroTileMode::Thick,

            _ => MicroTileMode::Display,
        }
    }

    fn array_mode(&self) -> ArrayMode {
        match self {
            TileMode::DisplayLinearAligned => ArrayMode::kLinearAligned,

            TileMode::Depth1dThin | TileMode::Display1dThin | TileMode::Thin1dThin => {
                ArrayMode::k1dTiledThin
            }

            TileMode::Thick1dThick => ArrayMode::k1dTiledThick,

            TileMode::Depth2dThin64
            | TileMode::Depth2dThin128
            | TileMode::Depth2dThin256
            | TileMode::Depth2dThin512
            | TileMode::Depth2dThin1k
            | TileMode::Display2dThin
            | TileMode::Thin2dThin => ArrayMode::k2dTiledThin,

            TileMode::DisplayThinPrt | TileMode::ThinThinPrt => ArrayMode::kTiledThinPrt,

            TileMode::Depth2dThinPrt256
            | TileMode::Depth2dThinPrt1k
            | TileMode::Display2dThinPrt
            | TileMode::Thin2dThinPrt => ArrayMode::k2dTiledThinPrt,

            TileMode::Thick2dThick => ArrayMode::k2dTiledThick,
            TileMode::Thick2dXthick => ArrayMode::k2dTiledXThick,

            TileMode::ThickThickPrt => ArrayMode::kTiledThickPrt,
            TileMode::Thick2dThickPrt => ArrayMode::k2dTiledThickPrt,
            TileMode::Thin3dThinPrt => ArrayMode::k3dTiledThinPrt,
            TileMode::Thin3dThin => ArrayMode::k3dTiledThin,
            TileMode::Thick3dThick => ArrayMode::k3dTiledThick,
            TileMode::Thick3dXthick => ArrayMode::k3dTiledXThick,
            TileMode::Thick3dThickPrt => ArrayMode::k3dTiledThickPrt,

            TileMode::DisplayLinearGeneral => ArrayMode::kLinearGeneral,
        }
    }

    fn pipe_config(&self) -> Option<PipeConfig> {
        match self {
            TileMode::DisplayThinPrt
            | TileMode::Thin3dThin
            | TileMode::ThinThinPrt
            | TileMode::Thick3dThick
            | TileMode::ThickThickPrt
            | TileMode::Thick3dThickPrt
            | TileMode::Thick3dXthick => Some(PipeConfig::P8_32x32_8x16),

            TileMode::DisplayLinearGeneral => None,

            _ => Some(PipeConfig::P8_32x32_16x16),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    // From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GpuAddress/GnmRegsinfoPrivate.h#L8-L20
    const ARRAY_MODE_MASK: u32 = 0x0000003c;
    const PIPE_CONFIG_MASK: u32 = 0x000007c0;
    const TILE_SPLIT_MASK: u32 = 0x00003800;
    const MICRO_TILE_MODE_MASK: u32 = 0x01c00000;
    const SAMPLE_SPLIT_MASK: u32 = 0x06000000;

    const ARRAY_MODE_SHIFT: u32 = 2;
    const PIPE_CONFIG_SHIFT: u32 = 6;
    const TILE_SPLIT_SHIFT: u32 = 11;
    const MICRO_TILE_MODE_SHIFT: u32 = 22;
    const SAMPLE_SPLIT_SHIFT: u32 = 25;

    #[test]
    fn verify_all_tile_modes() {
        // From https://github.com/Inori/GPCS4/blob/88480de36f983c66f7dd2c5951fead1a740d47a5/GPCS4/Graphics/Gnm/GpuAddress/GnmTilemodes.cpp#L44-L78
        let tile_mode_values = [
            0x90800310, // GB_TILE_MODE0  0x00 kTileModeDepth_2dThin_64
            0x90800B10, // GB_TILE_MODE1  0x01 kTileModeDepth_2dThin_128
            0x90801310, // GB_TILE_MODE2  0x02 kTileModeDepth_2dThin_256
            0x90801B10, // GB_TILE_MODE3  0x03 kTileModeDepth_2dThin_512
            0x90802310, // GB_TILE_MODE4  0x04 kTileModeDepth_2dThin_1K
            0x90800308, // GB_TILE_MODE5  0x05 kTileModeDepth_1dThin
            0x90801318, // GB_TILE_MODE6  0x06 kTileModeDepth_2dThinPrt_256
            0x90802318, // GB_TILE_MODE7  0x07 kTileModeDepth_2dThinPrt_1K
            0x90000304, // GB_TILE_MODE8  0x08 kTileModeDisplay_LinearAligned
            0x90000308, // GB_TILE_MODE9  0x09 kTileModeDisplay_1dThin
            0x92000310, // GB_TILE_MODE10 0x0A kTileModeDisplay_2dThin
            0x92000294, // GB_TILE_MODE11 0x0B kTileModeDisplay_ThinPrt
            0x92000318, // GB_TILE_MODE12 0x0C kTileModeDisplay_2dThinPrt
            0x90400308, // GB_TILE_MODE13 0x0D kTileModeThin_1dThin
            0x92400310, // GB_TILE_MODE14 0x0E kTileModeThin_2dThin
            0x924002B0, // GB_TILE_MODE15 0x0F kTileModeThin_3dThin
            0x92400294, // GB_TILE_MODE16 0x10 kTileModeThin_ThinPrt
            0x92400318, // GB_TILE_MODE17 0x11 kTileModeThin_2dThinPrt
            0x9240032C, // GB_TILE_MODE18 0x12 kTileModeThin_3dThinPrt
            0x9100030C, // GB_TILE_MODE19 0x13 kTileModeThick_1dThick
            0x9100031C, // GB_TILE_MODE20 0x14 kTileModeThick_2dThick
            0x910002B4, // GB_TILE_MODE21 0x15 kTileModeThick_3dThick
            0x910002A4, // GB_TILE_MODE22 0x16 kTileModeThick_ThickPrt
            0x91000328, // GB_TILE_MODE23 0x17 kTileModeThick_2dThickPrt
            0x910002BC, // GB_TILE_MODE24 0x18 kTileModeThick_3dThickPrt
            0x91000320, // GB_TILE_MODE25 0x19 kTileModeThick_2dXThick
            0x910002B8, // GB_TILE_MODE26 0x1A kTileModeThick_3dXThick
            0x90C00308, // GB_TILE_MODE27 0x1B kTileModeRotated_1dThin
            0x92C00310, // GB_TILE_MODE28 0x1C kTileModeRotated_2dThin
            0x92C00294, // GB_TILE_MODE29 0x1D kTileModeRotated_ThinPrt
            0x92C00318, // GB_TILE_MODE30 0x1E kTileModeRotated_2dThinPrt
            0x00000000, // GB_TILE_MODE31 0x1F kTileModeDisplay_LinearGeneral
        ];

        for tile_mode in TileMode::iter() {
            let raw_value = tile_mode_values[(tile_mode as u8) as usize];

            assert_eq!(
                Some(tile_mode.array_mode()),
                ArrayMode::from_repr(((raw_value & ARRAY_MODE_MASK) >> ARRAY_MODE_SHIFT) as _),
            );

            assert_eq!(
                tile_mode.pipe_config(),
                PipeConfig::from_repr(((raw_value & PIPE_CONFIG_MASK) >> PIPE_CONFIG_SHIFT) as _)
            );

            assert_eq!(
                Some(tile_mode.tile_split()),
                TileSplit::from_repr(((raw_value & TILE_SPLIT_MASK) >> TILE_SPLIT_SHIFT) as _)
            );

            assert_eq!(
                Some(tile_mode.micro_tile_mode()),
                MicroTileMode::from_repr(
                    ((raw_value & MICRO_TILE_MODE_MASK) >> MICRO_TILE_MODE_SHIFT) as _
                )
            );

            assert_eq!(
                Some(tile_mode.sample_split()),
                SampleSplit::from_repr(
                    ((raw_value & SAMPLE_SPLIT_MASK) >> SAMPLE_SPLIT_SHIFT) as _
                )
            );
        }
    }

    const BANK_WIDTH_MASK: u32 = 0x00000003;
    const BANK_HEIGHT_MASK: u32 = 0x0000000c;
    const MACRO_TILE_ASPECT_MASK: u32 = 0x00000030;
    const NUM_BANKS_MASK: u32 = 0x000000c0;

    const BANK_WIDTH_SHIFT: u32 = 0;
    const BANK_HEIGHT_SHIFT: u32 = 2;
    const MACRO_TILE_ASPECT_SHIFT: u32 = 4;
    const NUM_BANKS_SHIFT: u32 = 6;

    #[test]
    fn verify_all_macro_tile_modes() {
        let macro_tile_values = [
            0x26E8, // GB_MACROTILE_MODE0  0x00 kMacroTileMode_1x4_16
            0x26D4, // GB_MACROTILE_MODE1  0x01 kMacroTileMode_1x2_16
            0x21D0, // GB_MACROTILE_MODE2  0x02 kMacroTileMode_1x1_16
            0x21D0, // GB_MACROTILE_MODE3  0x03 kMacroTileMode_1x1_16_dup
            0x2080, // GB_MACROTILE_MODE4  0x04 kMacroTileMode_1x1_8
            0x2040, // GB_MACROTILE_MODE5  0x05 kMacroTileMode_1x1_4
            0x1000, // GB_MACROTILE_MODE6  0x06 kMacroTileMode_1x1_2
            0x0000, // GB_MACROTILE_MODE7  0x07 kMacroTileMode_1x1_2_dup
            0x36EC, // GB_MACROTILE_MODE8  0x08 kMacroTileMode_1x8_16
            0x26E8, // GB_MACROTILE_MODE9  0x09 kMacroTileMode_1x4_16_dup
            0x21D4, // GB_MACROTILE_MODE10 0x0A kMacroTileMode_1x2_16_dup
            0x20D0, // GB_MACROTILE_MODE11 0x0B kMacroTileMode_1x1_16_dup2
            0x1080, // GB_MACROTILE_MODE12 0x0C kMacroTileMode_1x1_8_dup
            0x1040, // GB_MACROTILE_MODE13 0x0D kMacroTileMode_1x1_4_dup
            0x0000, // GB_MACROTILE_MODE14 0x0E kMacroTileMode_1x1_2_dup2
            0x0000, // GB_MACROTILE_MODE15 0x0F kMacroTileMode_1x1_2_dup3
        ];

        for mode in MacroTileMode::iter() {
            let raw_value = macro_tile_values[mode as u8 as usize];
            let info = mode.bank_info(false);

            assert_eq!(
                info.width,
                ((raw_value & BANK_WIDTH_MASK) >> BANK_WIDTH_SHIFT) + 1,
            );

            assert_eq!(
                info.height,
                1 << ((raw_value & BANK_HEIGHT_MASK) >> BANK_HEIGHT_SHIFT),
            );

            assert_eq!(
                info.macro_tile_aspect,
                1 << ((raw_value & MACRO_TILE_ASPECT_MASK) >> MACRO_TILE_ASPECT_SHIFT),
            );

            assert_eq!(
                info.num_banks,
                2 << ((raw_value & NUM_BANKS_MASK) >> NUM_BANKS_SHIFT),
            );
        }
    }
}

#[derive(FromRepr, EnumIter, Debug, Clone, Copy)]
#[repr(u8)]
pub enum MacroTileMode {
    k1x4_16 = 0x00000000, //< bankWidth=1 bankHeight=4 macroTileAspect=4 numBanks=16 altBankHeight=4 altNumBanks= 8 altMacroTileAspect=2
    k1x2_16 = 0x00000001, //< bankWidth=1 bankHeight=2 macroTileAspect=2 numBanks=16 altBankHeight=4 altNumBanks= 8 altMacroTileAspect=2
    k1x1_16 = 0x00000002, //< bankWidth=1 bankHeight=1 macroTileAspect=2 numBanks=16 altBankHeight=2 altNumBanks= 8 altMacroTileAspect=1
    k1x1_16_dup = 0x00000003, //< bankWidth=1 bankHeight=1 macroTileAspect=2 numBanks=16 altBankHeight=2 altNumBanks= 8 altMacroTileAspect=1
    k1x1_8 = 0x00000004, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 8 altBankHeight=1 altNumBanks= 8 altMacroTileAspect=1
    k1x1_4 = 0x00000005, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 4 altBankHeight=1 altNumBanks= 8 altMacroTileAspect=1
    k1x1_2 = 0x00000006, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 2 altBankHeight=1 altNumBanks= 4 altMacroTileAspect=1
    k1x1_2_dup = 0x00000007, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 2 altBankHeight=1 altNumBanks= 2 altMacroTileAspect=1
    k1x8_16 = 0x00000008, //< bankWidth=1 bankHeight=8 macroTileAspect=4 numBanks=16 altBankHeight=4 altNumBanks=16 altMacroTileAspect=2
    k1x4_16_dup = 0x00000009, //< bankWidth=1 bankHeight=4 macroTileAspect=4 numBanks=16 altBankHeight=4 altNumBanks= 8 altMacroTileAspect=2
    k1x2_16_dup = 0x0000000A, //< bankWidth=1 bankHeight=2 macroTileAspect=2 numBanks=16 altBankHeight=2 altNumBanks= 8 altMacroTileAspect=1
    k1x1_16_dup2 = 0x0000000B, //< bankWidth=1 bankHeight=1 macroTileAspect=2 numBanks=16 altBankHeight=1 altNumBanks= 8 altMacroTileAspect=1
    k1x1_8_dup = 0x0000000C, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 8 altBankHeight=1 altNumBanks= 4 altMacroTileAspect=1
    k1x1_4_dup = 0x0000000D, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 4 altBankHeight=1 altNumBanks= 4 altMacroTileAspect=1
    k1x1_2_dup2 = 0x0000000E, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 2 altBankHeight=1 altNumBanks= 2 altMacroTileAspect=1
    k1x1_2_dup3 = 0x0000000F, //< bankWidth=1 bankHeight=1 macroTileAspect=1 numBanks= 2 altBankHeight=1 altNumBanks= 2 altMacroTileAspect=1
}

const DRAM_ROW_SIZE: u32 = 0x400;

impl MacroTileMode {
    pub fn from(tile_mode: TileMode, surface_format: SurfaceFormat) -> Option<MacroTileMode> {
        let array_mode = tile_mode.array_mode();
        let micro_tile_mode = tile_mode.micro_tile_mode();
        let sample_split = tile_mode.sample_split();
        let tile_split = tile_mode.tile_split();

        let bits_per_element = (surface_format.bytes_len() * 8) as u32;
        let num_fragments_per_pixel = 1;

        let tile_thickness = array_mode.thickness();
        let tile_bytes_1x = bits_per_element * 8 * 8 * tile_thickness / 8;
        let sample_split = 1u32 << (sample_split as u8);
        let color_tile_split = 256u32.max(sample_split * tile_bytes_1x);
        let tile_split = if micro_tile_mode == MicroTileMode::Depth {
            64u32 << (tile_split as u8)
        } else {
            color_tile_split as _
        };

        let tile_split_c = min(DRAM_ROW_SIZE, tile_split);
        let tile_bytes = min(tile_split_c, num_fragments_per_pixel * tile_bytes_1x);

        let mtm_index = tile_bytes / 64;
        let raw_value = if array_mode.is_partially_resident_texture() {
            mtm_index + 8
        } else {
            mtm_index
        };

        MacroTileMode::from_repr(raw_value as _)
    }
}

pub struct BankInfo {
    pub width: u32,
    pub height: u32,
    pub macro_tile_aspect: u32,
    pub num_banks: u32,
}

impl MacroTileMode {
    pub const fn bank_info(&self, alt: bool) -> BankInfo {
        let width = 1;

        match (self, alt) {
            (Self::k1x4_16, true) => BankInfo {
                width,
                height: 4,
                macro_tile_aspect: 2,
                num_banks: 8,
            },
            (Self::k1x4_16, false) => BankInfo {
                width,
                height: 4,
                macro_tile_aspect: 4,
                num_banks: 16,
            },
            (Self::k1x2_16, true) => BankInfo {
                width,
                height: 4,
                macro_tile_aspect: 2,
                num_banks: 8,
            },
            (Self::k1x2_16, false) => BankInfo {
                width,
                height: 2,
                macro_tile_aspect: 2,
                num_banks: 16,
            },
            (Self::k1x1_16 | Self::k1x1_16_dup, true) => BankInfo {
                width,
                height: 2,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x1_16 | Self::k1x1_16_dup, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 2,
                num_banks: 16,
            },
            (Self::k1x1_8, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x1_8, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x1_4, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x1_4, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 4,
            },
            (Self::k1x1_2 | Self::k1x1_2_dup, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 4,
            },
            (Self::k1x1_2 | Self::k1x1_2_dup, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 2,
            },
            (Self::k1x8_16, true) => BankInfo {
                width,
                height: 4,
                macro_tile_aspect: 2,
                num_banks: 16,
            },
            (Self::k1x8_16, false) => BankInfo {
                width,
                height: 8,
                macro_tile_aspect: 4,
                num_banks: 16,
            },
            (Self::k1x4_16_dup, true) => BankInfo {
                width,
                height: 4,
                macro_tile_aspect: 2,
                num_banks: 8,
            },
            (Self::k1x4_16_dup, false) => BankInfo {
                width,
                height: 4,
                macro_tile_aspect: 4,
                num_banks: 16,
            },
            (Self::k1x2_16_dup, true) => BankInfo {
                width,
                height: 2,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x2_16_dup, false) => BankInfo {
                width,
                height: 2,
                macro_tile_aspect: 2,
                num_banks: 16,
            },
            (Self::k1x1_16_dup2, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x1_16_dup2, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 2,
                num_banks: 16,
            },
            (Self::k1x1_8_dup, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 4,
            },
            (Self::k1x1_8_dup, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 8,
            },
            (Self::k1x1_4_dup, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 4,
            },
            (Self::k1x1_4_dup, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 4,
            },
            (Self::k1x1_2_dup2 | Self::k1x1_2_dup3, true) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 2,
            },
            (Self::k1x1_2_dup2 | Self::k1x1_2_dup3, false) => BankInfo {
                width,
                height: 1,
                macro_tile_aspect: 1,
                num_banks: 2,
            },
        }
    }
}

const fn fast_int_log2(x: u32) -> u32 {
    31 - x.leading_zeros()
}

impl BankInfo {
    pub fn index(&self, x: u32, y: u32, num_pipes: u32) -> u32 {
        let x_shift_offset = fast_int_log2(self.width * num_pipes);
        let y_shift_offset = fast_int_log2(self.height);
        let xs = x >> x_shift_offset;
        let ys = y >> y_shift_offset;

        match self.num_banks {
            2 => ((xs >> 3) ^ (ys >> 3)) & 0x1,
            4 => {
                let mut bank = ((xs >> 3) ^ (ys >> 4)) & 0x1;
                bank |= (((xs >> 4) ^ (ys >> 3)) & 0x1) << 1;
                bank
            }
            8 => {
                let mut bank = ((xs >> 3) ^ (ys >> 5)) & 0x1;
                bank |= (((xs >> 4) ^ (ys >> 4) ^ (ys >> 5)) & 0x1) << 1;
                bank |= (((xs >> 5) ^ (ys >> 3)) & 0x1) << 2;
                bank
            }
            16 => {
                let mut bank = ((xs >> 3) ^ (ys >> 6)) & 0x1;
                bank |= (((xs >> 4) ^ (ys >> 5) ^ (ys >> 6)) & 0x1) << 1;
                bank |= (((xs >> 5) ^ (ys >> 4)) & 0x1) << 2;
                bank |= (((xs >> 6) ^ (ys >> 3)) & 0x1) << 3;
                bank
            }
            _ => unreachable!(
                "invalid num_banks ({}) -- must be 2, 4, 8, or 16.",
                self.num_banks
            ),
        }
    }
}

fn get_element_index(
    x: u32,
    y: u32,
    z: u32,
    bits_per_element: u32,
    micro_tile_mode: MicroTileMode,
    array_mode: ArrayMode,
) -> u32 {
    let mut elem = 0;

    match micro_tile_mode {
        MicroTileMode::Display => match bits_per_element {
            8 => {
                elem |= ((x >> 0) & 0x1) << 0;
                elem |= ((x >> 1) & 0x1) << 1;
                elem |= ((x >> 2) & 0x1) << 2;
                elem |= ((y >> 1) & 0x1) << 3;
                elem |= ((y >> 0) & 0x1) << 4;
                elem |= ((y >> 2) & 0x1) << 5;
            }
            16 => {
                elem |= ((x >> 0) & 0x1) << 0;
                elem |= ((x >> 1) & 0x1) << 1;
                elem |= ((x >> 2) & 0x1) << 2;
                elem |= ((y >> 0) & 0x1) << 3;
                elem |= ((y >> 1) & 0x1) << 4;
                elem |= ((y >> 2) & 0x1) << 5;
            }
            32 => {
                elem |= ((x >> 0) & 0x1) << 0;
                elem |= ((x >> 1) & 0x1) << 1;
                elem |= ((y >> 0) & 0x1) << 2;
                elem |= ((x >> 2) & 0x1) << 3;
                elem |= ((y >> 1) & 0x1) << 4;
                elem |= ((y >> 2) & 0x1) << 5;
            }
            64 => {
                elem |= ((x >> 0) & 0x1) << 0;
                elem |= ((y >> 0) & 0x1) << 1;
                elem |= ((x >> 1) & 0x1) << 2;
                elem |= ((x >> 2) & 0x1) << 3;
                elem |= ((y >> 1) & 0x1) << 4;
                elem |= ((y >> 2) & 0x1) << 5;
            }
            _ => panic!(
                "Unsupported bitsPerElement ({}) for displayable surface.",
                bits_per_element
            ),
        },
        MicroTileMode::Thin | MicroTileMode::Depth => {
            elem |= ((x >> 0) & 0x1) << 0;
            elem |= ((y >> 0) & 0x1) << 1;
            elem |= ((x >> 1) & 0x1) << 2;
            elem |= ((y >> 1) & 0x1) << 3;
            elem |= ((x >> 2) & 0x1) << 4;
            elem |= ((y >> 2) & 0x1) << 5;

            // Handle Z for Thick/XThick array modes
            match array_mode {
                ArrayMode::k2dTiledXThick | ArrayMode::k3dTiledXThick => {
                    elem |= ((z >> 2) & 0x1) << 8;
                    elem |= ((z >> 0) & 0x1) << 6;
                    elem |= ((z >> 1) & 0x1) << 7;
                }
                ArrayMode::k1dTiledThick
                | ArrayMode::k2dTiledThick
                | ArrayMode::k3dTiledThick
                | ArrayMode::kTiledThickPrt
                | ArrayMode::k2dTiledThickPrt
                | ArrayMode::k3dTiledThickPrt => {
                    elem |= ((z >> 0) & 0x1) << 6;
                    elem |= ((z >> 1) & 0x1) << 7;
                }
                _ => {}
            }
        }
        MicroTileMode::Thick => {
            match array_mode {
                ArrayMode::k2dTiledXThick | ArrayMode::k3dTiledXThick => {
                    elem |= ((z >> 2) & 0x1) << 8;
                }
                ArrayMode::k1dTiledThick
                | ArrayMode::k2dTiledThick
                | ArrayMode::k3dTiledThick
                | ArrayMode::kTiledThickPrt
                | ArrayMode::k2dTiledThickPrt
                | ArrayMode::k3dTiledThickPrt => {}
                _ => unreachable!(
                    "Invalid arrayMode ({:?}) for thick/xthick microTileMode=kMicroTileModeThick.",
                    array_mode
                ),
            }

            match bits_per_element {
                8 | 16 => {
                    elem |= ((x >> 0) & 0x1) << 0;
                    elem |= ((y >> 0) & 0x1) << 1;
                    elem |= ((x >> 1) & 0x1) << 2;
                    elem |= ((y >> 1) & 0x1) << 3;
                    elem |= ((z >> 0) & 0x1) << 4;
                    elem |= ((z >> 1) & 0x1) << 5;
                    elem |= ((x >> 2) & 0x1) << 6;
                    elem |= ((y >> 2) & 0x1) << 7;
                }
                32 => {
                    elem |= ((x >> 0) & 0x1) << 0;
                    elem |= ((y >> 0) & 0x1) << 1;
                    elem |= ((x >> 1) & 0x1) << 2;
                    elem |= ((z >> 0) & 0x1) << 3;
                    elem |= ((y >> 1) & 0x1) << 4;
                    elem |= ((z >> 1) & 0x1) << 5;
                    elem |= ((x >> 2) & 0x1) << 6;
                    elem |= ((y >> 2) & 0x1) << 7;
                }
                64 | 128 => {
                    elem |= ((x >> 0) & 0x1) << 0;
                    elem |= ((y >> 0) & 0x1) << 1;
                    elem |= ((z >> 0) & 0x1) << 2;
                    elem |= ((x >> 1) & 0x1) << 3;
                    elem |= ((y >> 1) & 0x1) << 4;
                    elem |= ((z >> 1) & 0x1) << 5;
                    elem |= ((x >> 2) & 0x1) << 6;
                    elem |= ((y >> 2) & 0x1) << 7;
                }
                _ => unreachable!(
                    "Invalid bitsPerElement ({}) for microTileMode=kMicroTileModeThick.",
                    bits_per_element
                ),
            }
        }
        MicroTileMode::Rotated => unreachable!("Rotated micro tile mode not supported"),
    }

    elem
}

const BANK_INTERLEAVE: u32 = 1;
const MICRO_TILE_WIDTH: u32 = 8;
const MICRO_TILE_HEIGHT: u32 = 8;

pub fn get_tiled_element_bit_offset(
    x: u32,
    y: u32,
    z: u32,
    fragment_index: u32,
    bits_per_element: u32,
    micro_tile_mode: MicroTileMode,
    array_mode: ArrayMode,
    pipe_config: PipeConfig,
    macro_tile_mode: MacroTileMode,
    tile_split_bytes: u32,
    pipe_interleave_bytes: u32,
    num_fragments_per_pixel: u32,
    macro_tile_width: u32,
    macro_tile_height: u32,
    padded_width: u32,
    padded_height: u32,
    array_slice: u32,
    bank_swizzle_mask: u32,
    pipe_swizzle_mask: u32,
) -> u64 {
    let element_index = get_element_index(x, y, z, bits_per_element, micro_tile_mode, array_mode);

    // Handle PRT modes
    let (xh, yh) = match array_mode {
        ArrayMode::kTiledThinPrt | ArrayMode::kTiledThickPrt => {
            (x % macro_tile_width, y % macro_tile_height)
        }
        _ => (x, y),
    };

    let pipe = pipe_config.index(xh, yh);
    let bank_info = macro_tile_mode.bank_info(false);
    let bank = bank_info.index(xh, yh, pipe_config.count());

    let tile_bytes = (MICRO_TILE_WIDTH
        * MICRO_TILE_HEIGHT
        * array_mode.thickness()
        * bits_per_element
        * num_fragments_per_pixel
        + 7)
        / 8;

    let element_offset = match micro_tile_mode {
        MicroTileMode::Depth => {
            let pixel_offset =
                element_index as u64 * bits_per_element as u64 * num_fragments_per_pixel as u64;
            pixel_offset + (fragment_index as u64 * bits_per_element as u64)
        }
        _ => {
            let fragment_offset =
                fragment_index as u64 * (tile_bytes as u64 / num_fragments_per_pixel as u64) * 8;
            fragment_offset + (element_index as u64 * bits_per_element as u64)
        }
    };

    // Handle tile splitting
    let (slices_per_tile, tile_split_slice, element_offset, tile_bytes) =
        if tile_bytes > tile_split_bytes && array_mode.thickness() == 1 {
            let slices = tile_bytes / tile_split_bytes;
            let split_slice = element_offset / (tile_split_bytes as u64 * 8);
            (
                slices,
                split_slice,
                element_offset % (tile_split_bytes as u64 * 8),
                tile_split_bytes,
            )
        } else {
            (1, 0, element_offset, tile_bytes)
        };

    // Verify tiling restrictions
    if BANK_INTERLEAVE * pipe_interleave_bytes
        > min(tile_split_bytes, tile_bytes) * bank_info.width * bank_info.height
    {
        unreachable!("internal tiling error: bank interleave check failed");
    }
    if BANK_INTERLEAVE * pipe_interleave_bytes
        > pipe_config.count()
            * bank_info.width
            * bank_info.macro_tile_aspect
            * std::cmp::min(tile_split_bytes, tile_bytes)
    {
        unreachable!("internal tiling error: pipe interleave check failed");
    }

    let macro_tile_bytes = (macro_tile_width / MICRO_TILE_WIDTH)
        * (macro_tile_height / MICRO_TILE_HEIGHT)
        * tile_bytes
        / (pipe_config.count() * bank_info.num_banks);
    let macro_tiles_per_row = padded_width / macro_tile_width;
    let macro_tile_index = (y / macro_tile_height) * macro_tiles_per_row + (x / macro_tile_width);
    let macro_tile_offset = macro_tile_index * macro_tile_bytes;
    let macro_tiles_per_slice = macro_tiles_per_row * (padded_height / macro_tile_height);
    let slice_bytes = macro_tiles_per_slice * macro_tile_bytes;

    if z != 0 && array_slice != 0 {
        unreachable!("arrays of volume textures aren't supported");
    }

    let mut slice = z;
    let slice_offset =
        (tile_split_slice + slices_per_tile * slice / array_mode.thickness()) * slice_bytes;
    if array_slice != 0 {
        slice = array_slice;
    }

    let tile_row_index = (y / MICRO_TILE_HEIGHT) % bank_info.height;
    let tile_column_index = ((x / MICRO_TILE_WIDTH) / pipe_config.count()) % bank_info.width;
    let tile_index = tile_row_index * bank_info.width + tile_column_index;
    let tile_offset = tile_index * tile_bytes;

    // Handle pipe/bank swizzling and rotation
    let mut pipe = pipe ^ pipe_swizzle_mask;
    let mut pipe_slice_rotation = 0;
    match array_mode {
        ArrayMode::k3dTiledThin | ArrayMode::k3dTiledThick | ArrayMode::k3dTiledXThick => {
            pipe_slice_rotation =
                std::cmp::max(1, (pipe_config.count() / 2) - 1) * (slice / array_mode.thickness());
        }
        _ => {}
    }
    pipe = (pipe + pipe_slice_rotation) % pipe_config.count();

    let mut slice_rotation = 0;
    match array_mode {
        ArrayMode::k2dTiledThin | ArrayMode::k2dTiledThick | ArrayMode::k2dTiledXThick => {
            slice_rotation = ((bank_info.num_banks / 2) - 1) * (slice / array_mode.thickness());
        }
        ArrayMode::k3dTiledThin | ArrayMode::k3dTiledThick | ArrayMode::k3dTiledXThick => {
            slice_rotation = std::cmp::max(1, (pipe_config.count() / 2) - 1)
                * (slice / array_mode.thickness())
                / pipe_config.count();
        }
        _ => {}
    }

    let mut tile_split_slice_rotation = 0;
    match array_mode {
        ArrayMode::k2dTiledThin
        | ArrayMode::k3dTiledThin
        | ArrayMode::k2dTiledThinPrt
        | ArrayMode::k3dTiledThinPrt => {
            tile_split_slice_rotation = ((bank_info.num_banks / 2) + 1) * tile_split_slice as u32;
        }
        _ => {}
    }

    let bank = (bank ^ bank_swizzle_mask ^ slice_rotation ^ tile_split_slice_rotation)
        % bank_info.num_banks;

    let total_offset = (slice_offset + macro_tile_offset + tile_offset as u64) * 8 + element_offset;
    let bit_offset = total_offset & 0x7;
    let total_offset = total_offset >> 3;

    let pipe_interleave_mask = (1u64 << pipe_interleave_bytes.trailing_zeros()) - 1;
    let pipe_interleave_offset = total_offset & pipe_interleave_mask;
    let offset = total_offset >> pipe_interleave_bytes.trailing_zeros();

    let final_byte_offset = pipe_interleave_offset
        | ((pipe as u64) << pipe_interleave_bytes.trailing_zeros())
        | ((bank as u64)
            << (pipe_interleave_bytes.trailing_zeros() + pipe_config.count().trailing_zeros()))
        | (offset
            << (pipe_interleave_bytes.trailing_zeros()
                + pipe_config.count().trailing_zeros()
                + bank_info.num_banks.trailing_zeros()));

    (final_byte_offset << 3) | bit_offset
}
