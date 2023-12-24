mod generated;

use anyhow::format_err;
use byteorder::{LittleEndian, ReadBytesExt};
pub use generated::{OpCode, OpFormat, OpInfo, OPS};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

impl OpCode {
    pub fn op_info(&self) -> &'static OpInfo {
        OPS[self]
    }
}

impl OpFormat {
    pub fn ops(&self) -> impl Iterator<Item = OpInfo> {
        OPS.iter().filter(|it| &it.format == self)
    }

    pub fn op_mask(&self) -> u32 {
        fn mask(start_bit_idx: u32, len_bits: u32) -> u32 {
            ((1 << len_bits) - 1) << (32 - len_bits - start_bit_idx)
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

    pub pattern: Option<OpFormatPattern>,
    // pub op_start_bit_idx: u8,
    // pub op_len_bits: u8,
}

struct OpFormatPattern {
    width: u8,
    bits: u32,
}

#[macro_export]
macro_rules! op_format_pattern {
    ($bitstring:expr) => {{
        const WIDTH: usize = $bitstring.len();
        const BITS: u32 = {
            let bytes = $bitstring.as_bytes();
            let mut bits = 0;

            let mut i = 0;
            while i < WIDTH {
                bits <<= 1;
                match bytes[i] {
                    b'0' => {}
                    b'1' => bits |= 1,
                    _ => panic!("Invalid character in bitstring"), // Causes compile-time error
                }
                i += 1;
            }
            bits
        };

        OpFormatPattern {
            width: WIDTH as u8,
            bits: BITS,
        }
    }};
}

pub const OP_FORMATS: [OpFormatInfo; 25] = [
    OpFormatInfo {
        format: OpFormat::SOP1,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("101111101")),
        // op_start_bit_idx: 16,
        // op_len_bits: 8,
    },
    OpFormatInfo {
        format: OpFormat::SOP2,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("10")),
        // op_start_bit_idx: 0,
        // op_len_bits: 0,
    },
    OpFormatInfo {
        format: OpFormat::SOPK,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("1011")),
    },
    OpFormatInfo {
        format: OpFormat::SOPP,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("101111111")),
    },
    OpFormatInfo {
        format: OpFormat::SOPC,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("101111100")),
    },
    OpFormatInfo {
        format: OpFormat::SMEM,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("11000")),
    },
    OpFormatInfo {
        format: OpFormat::DS,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("110110")),
    },
    OpFormatInfo {
        format: OpFormat::LDSDIR,
        bytes_len: 8,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::MTBUF,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111010")),
    },
    OpFormatInfo {
        format: OpFormat::MUBUF,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111000")),
    },
    OpFormatInfo {
        format: OpFormat::MIMG,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111100")),
    },
    OpFormatInfo {
        format: OpFormat::EXP,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111110")),
    },
    OpFormatInfo {
        format: OpFormat::FLAT,
        bytes_len: 8,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::GLOBAL,
        bytes_len: 8,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::SCRATCH,
        bytes_len: 8,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::VINTRP,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("110010")),
    },
    OpFormatInfo {
        format: OpFormat::VINTERP_INREG,
        bytes_len: 8,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::VOP1,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("0111111")),
    },
    OpFormatInfo {
        format: OpFormat::VOP2,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("0")),
    },
    OpFormatInfo {
        format: OpFormat::VOPC,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("0111110")),
    },
    OpFormatInfo {
        format: OpFormat::VOP3,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("110100")),
    },
    OpFormatInfo {
        format: OpFormat::VOP3P,
        bytes_len: 8,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::SDWA,
        bytes_len: 4,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::DPP16,
        bytes_len: 4,
        pattern: None,
    },
    OpFormatInfo {
        format: OpFormat::DPP8,
        bytes_len: 8,
        pattern: None,
    },
];

struct OpFormatMatcher {
    groups: BTreeMap<u8, Vec<(OpFormat, u32)>>,
}

impl OpFormatMatcher {
    pub fn new(formats: &[OpFormatInfo]) -> OpFormatMatcher {
        let groups = {
            let mut groups: BTreeMap<u8, Vec<(OpFormat, u32)>> = BTreeMap::new();

            for info in formats {
                if let Some(it) = info.pattern {
                    groups
                        .entry(it.width)
                        .or_insert_with(Vec::new)
                        .push((info.format, it.bits));
                }
            }

            groups
        };

        OpFormatMatcher { groups }
    }

    pub fn from(&self, token: u32) -> Option<OpFormat> {
        fn bitmask(size: u32) -> u32 {
            ((1 << size) - 1) << (32 - size)
        }

        fn extract_pattern(token: u32, size: u32) -> u32 {
            (token & bitmask(size)) >> (32 - size)
        }

        for (width, values) in self.groups.iter().rev() {
            let masked = extract_pattern(token, width);

            for (format, pattern) in values {
                if masked == pattern {
                    return Some(format);
                }
            }
        }

        return None;
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
        let format =
            OpFormat::matches(token).ok_or_else(|| format_err!("unknown token {:x}", token))?;
    }
}
