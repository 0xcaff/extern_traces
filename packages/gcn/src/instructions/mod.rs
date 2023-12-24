mod generated;

pub use generated::{OpCode, OpFormat, OpInfo, OPS};
use std::io::Read;
use anyhow::format_err;
use byteorder::{LittleEndian, ReadBytesExt};

impl OpCode {
    pub fn op_info(&self) -> &'static OpInfo {
        OPS[self]
    }
}

impl OpFormat {
    pub fn ops(&self) -> impl Iterator<Item = OpInfo> {
        OPS.iter().filter(|it| &it.format == self)
    }

    pub fn bytes_len(&self) -> usize {
        match self {
            OpFormat::SOP2 | OpFormat::SOPK | OpFormat::SOP1 | OpFormat::SOPC | OpFormat::SOPP => 4,

            OpFormat::VOP2 | OpFormat::VOP1 | OpFormat::VOPC => 4,

            OpFormat::VOP3 => 8,

            OpFormat::MUBUF | OpFormat::MTBUF | OpFormat::MIMG => 8,

            OpFormat::VINTRP => 4,

            OpFormat::EXP | OpFormat::GLOBAL | OpFormat::SCRATCH => 8,

            OpFormat::DS => 8,

            OpFormat::SMEM => 4,
            OpFormat::LDSDIR => 8,

            OpFormat::FLAT => 8,

            OpFormat::SDWA => 4,
            OpFormat::DPP16 => 4,
            OpFormat::DPP8 => 8,

            OpFormat::VOP3P => 8,
            OpFormat::VINTERP_INREG => 8,
        }
    }

    pub fn op_mask(&self) -> u32 {
        fn mask(start_bit_idx: u32, len_bits: u32) -> u32 {
           ((1 << len_bits) - 1)  << (32 - len_bits - start_bit_idx)
        }

        match self {
            OpFormat::SOP2 => mask(2, 7),
            OpFormat::SOPK => mask(4, 5),
            OpFormat::SOP1 => mask(16, 8),
            OpFormat::SOPC => mask(9, 7),
            OpFormat::VOP2 => mask(1, 6),
            OpFormat::VOP1 => mask(15, 8),
            OpFormat::VOPC => mask(7, 8),
            OpFormat::VOP3 => mask(6, 9),
            OpFormat::SOPP => mask(9, 7),
            OpFormat::MUBUF => mask(7, 7),
            OpFormat::MTBUF => mask(13, 3),
            OpFormat::MIMG => mask(7, 7),
            OpFormat::DS => mask(6, 8),
            OpFormat::VINTRP => mask(14, 2),
        }
    }
}

struct OpFormatInfo {
    pub format: OpFormat,
    pub bytes_len: u8,

    pub pattern_width: u8,
    pub pattern: u32,

    pub op_start_bit_idx: u8,
    pub op_len_bits: u8,
}

pub static OP_FORMATS: [OpFormatInfo] = [
    // todo: fill this out
]


impl OpFormat {
    pub fn matches(token: u32) -> Option<OpFormat> {
        fn bitmask(size: u32) -> u32 {
            ((1 << size) - 1) << (32 - size)
        }

        fn shift(mask: u32, size: u32) -> u32 {
            mask << (32 - size)
        }

        fn extract_pattern(token: u32, size: u32) -> u32 {
            token & bitmask(size) >> (32 - size)
        }

        {
            let masked = extract_pattern(token, 9);

            match masked {
                0b101111101 => return Some(OpFormat::SOP1),
                0b101111100 => return Some(OpFormat::SOPC),
                0b101111111 => return Some(OpFormat::SOPP),
                _ => {}
            }
        }

        {
            let masked = extract_pattern(token, 7);

            match masked {
                0b0111111 => return Some(OpFormat::VOP1),
                0b0111110 => return Some(OpFormat::VOPC),
                _ => {}
            }
        }

        {
            let masked = extract_pattern(token, 6);

            match masked {
                0b111100 => return Some(OpFormat::MIMG),
                0b110110 => return Some(OpFormat::DS),
                0b110010 => return Some(OpFormat::VINTRP),
                0b111110 => return Some(OpFormat::EXP),
                0b110100 => return Some(OpFormat::VOP3),
                0b111000 => return Some(OpFormat::MUBUF),
                0b111010 => return Some(OpFormat::MTBUF),
                _ => {}
            }
        }

        {
            let masked = extract_pattern(token, 5);
            match masked {
                0b11000 => return Some(OpFormat::SMEM),
                _ => {}
            }
        }

        {
            let masked = extract_pattern(token, 4);
            match masked {
                0b1011 => return Some(OpFormat::SOPK),
                _ => {}
            }
        }

        {
            let masked = extract_pattern(token, 2);
            match masked {
                0b10 => return Some(OpFormat::SOP2),
                _ => {}
            }
        }

        {
            let masked = extract_pattern(token, 1);
            match masked {
                0b0 => return Some(OpFormat::VOP2),
                _ => {}
            }
        }

        None
    }
}

pub enum GfxLevel {
    GFX7,
    GFX9,
    GFX10,
    GFX11,
}

impl GfxLevel {
    pub fn ops(&self) -> &'static [Option<OpCode>] {
        &*match self {
            GfxLevel::GFX7 => generated::GFX7_OPS,
            GfxLevel::GFX9 => generated::GFX9_OPS,
            GfxLevel::GFX10 => generated::GFX10_OPS,
            GfxLevel::GFX11 => generated::GFX11_OPS,
        }
    }
}

pub struct Decoder<'a, R> {
    level: GfxLevel,
    ops: &'static [Option<OpCode>],

    reader: R,
}

pub struct Instruction {
    pub code: OpCode,
}

impl<R> Decoder<R>
where
    R: Read,
{
    pub fn new(level: GfxLevel, reader: R) -> Decoder<R> {
        let ops = level.ops();

        Decoder { level, ops, reader }
    }

    pub fn decode(&mut self) -> Result<Instruction, anyhow::Error> {
        let token = self.reader.read_u32::<LittleEndian>()?;
        let format = OpFormat::matches(token).ok_or_else(|| format_err!("unknown token {:x}", token))?;

        format.ops()

        OpFormat::

    }
}
