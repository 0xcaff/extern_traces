mod generated;

use anyhow::format_err;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::io::Read;

pub use generated::{OpCode, OpFormat, OpInfo, OPS};

impl OpCode {
    pub fn op_info(&self) -> &'static OpInfo {
        OPS[self]
    }
}

impl OpFormat {
    pub fn ops(&self) -> impl Iterator<Item = OpInfo> {
        OPS.iter().filter(|it| &it.format == self)
    }

    pub fn info(&self) -> &OpFormatInfo {
        &OP_FORMATS[self]
    }
}

struct OpFormatInfo {
    pub format: OpFormat,
    pub bytes_len: u8,

    pub pattern: Option<OpFormatPattern>,
    pub op_bits: Option<BitRange>,
}

struct OpFormatPattern {
    width: u8,
    bits: u32,
}

struct BitRange {
    start: u8,
    len: u8,
}

impl BitRange {
    pub fn of(&self, value: u32) -> u32 {
        let offset = 32 - self.len - self.start;

        let mask = ((1 << self.len) - 1) << offset;

        (mask & value) >> offset;
    }
}

pub const fn bitrange(start: u8, len: u8) -> BitRange {
    BitRange { start, len }
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
        op_bits: Some(bitrange(16, 8)),
    },
    OpFormatInfo {
        format: OpFormat::SOP2,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("10")),
        op_bits: Some(bitrange(2, 7)),
    },
    OpFormatInfo {
        format: OpFormat::SOPK,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("1011")),
        op_bits: Some(bitrange(4, 5)),
    },
    OpFormatInfo {
        format: OpFormat::SOPP,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("101111111")),
        op_bits: Some(bitrange(9, 7)),
    },
    OpFormatInfo {
        format: OpFormat::SOPC,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("101111100")),
        op_bits: Some(bitrange(9, 7)),
    },
    OpFormatInfo {
        format: OpFormat::SMEM,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("11000")),
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::DS,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("110110")),
        op_bits: Some(bitrange(6, 8)),
    },
    OpFormatInfo {
        format: OpFormat::LDSDIR,
        bytes_len: 8,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::MTBUF,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111010")),
        op_bits: Some(bitrange(13, 3)),
    },
    OpFormatInfo {
        format: OpFormat::MUBUF,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111000")),
        op_bits: Some(bitrange(7, 7)),
    },
    OpFormatInfo {
        format: OpFormat::MIMG,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111100")),
        op_bits: Some(bitrange(7, 7)),
    },
    OpFormatInfo {
        format: OpFormat::EXP,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("111110")),
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::FLAT,
        bytes_len: 8,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::GLOBAL,
        bytes_len: 8,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::SCRATCH,
        bytes_len: 8,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::VINTRP,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("110010")),
        op_bits: Some(bitrange(14, 2)),
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
        op_bits: Some(bitrange(15, 8)),
    },
    OpFormatInfo {
        format: OpFormat::VOP2,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("0")),
        op_bits: Some(bitrange(1, 6)),
    },
    OpFormatInfo {
        format: OpFormat::VOPC,
        bytes_len: 4,
        pattern: Some(op_format_pattern!("0111110")),
        op_bits: Some(bitrange(7, 8)),
    },
    OpFormatInfo {
        format: OpFormat::VOP3,
        bytes_len: 8,
        pattern: Some(op_format_pattern!("110100")),
        op_bits: Some(bitrange(6, 9)),
    },
    OpFormatInfo {
        format: OpFormat::VOP3P,
        bytes_len: 8,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::SDWA,
        bytes_len: 4,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::DPP16,
        bytes_len: 4,
        pattern: None,
        op_bits: None,
    },
    OpFormatInfo {
        format: OpFormat::DPP8,
        bytes_len: 8,
        pattern: None,
        op_bits: None,
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

impl OpInfo {
    pub fn op_code_for_level(&self, gfx_level: GfxLevel) -> Option<u16> {
        match gfx_level {
            GfxLevel::GFX7 => self.gfx7,
            GfxLevel::GFX9 => self.gfx9,
            GfxLevel::GFX10 => self.gfx10,
            GfxLevel::GFX11 => self.gfx11,
        }
    }
}

pub struct Decoder<'a, R> {
    level: GfxLevel,
    ops: Vec<OpCode>,

    op_format_matcher: OpFormatMatcher,

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
        let ops = level
            .ops()
            .iter()
            .filter_map(|it| *it)
            .collect::<Vec<OpCode>>();

        Decoder {
            level,
            ops,
            reader,
            op_format_matcher: OpFormatMatcher::new(&OP_FORMATS),
        }
    }

    pub fn decode(&mut self) -> Result<Instruction, anyhow::Error> {
        let token = self.reader.read_u32::<LittleEndian>()?;

        let format = self
            .op_format_matcher
            .from(token)
            .ok_or_else(|| format_err!("unknown token {:x}", token))?;

        let bitrange = format
            .info()
            .op_bits
            .ok_or_else(|| format_err!("unknown op_bits for format {}", format))?;

        let op_bits = bitrange.of(token) as u16;

        let _code = format
            .ops()
            .find_map(|op| {
                let op_code = op.op_code_for_level(self.level)?;

                if op_code == op_bits {
                    return Some(op.op_code);
                }

                None
            })
            .ok_or_else(|| format_err!("no opcode found"))?;

        Ok(Instruction { code })
    }
}
