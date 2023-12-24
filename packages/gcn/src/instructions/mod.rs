mod format_info;
mod generated;

use anyhow::format_err;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::io::Read;

use crate::instructions::format_info::OP_FORMATS;
pub use generated::{OpCode, OpFormat, OpInfo, OPS};

impl OpCode {
    pub fn op_info(&self) -> &'static OpInfo {
        &OPS[(*self) as usize]
    }
}

struct OpFormatMatcher {
    groups: BTreeMap<u8, Vec<(OpFormat, u32)>>,
}

impl OpFormatMatcher {
    pub fn new() -> OpFormatMatcher {
        let groups = {
            let mut groups: BTreeMap<u8, Vec<(OpFormat, u32)>> = BTreeMap::new();

            for info in OP_FORMATS {
                if let Some(it) = &info.pattern {
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
        fn bitmask(size: u8) -> u32 {
            ((1 << size) - 1) << (32 - size)
        }

        fn extract_pattern(token: u32, size: u8) -> u32 {
            (token & bitmask(size)) >> (32 - size)
        }

        for (width, values) in self.groups.iter().rev() {
            let masked = extract_pattern(token, *width);

            for (format, pattern) in values {
                if masked == *pattern {
                    return Some(*format);
                }
            }
        }

        return None;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GfxLevel {
    GFX7,
    GFX9,
    GFX10,
    GFX11,
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

pub struct Decoder<R> {
    level: GfxLevel,

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
        Decoder {
            level,
            reader,
            op_format_matcher: OpFormatMatcher::new(),
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
            .as_ref()
            .ok_or_else(|| format_err!("unknown op_bits for format {:?}", format))?;

        let op_bits = bitrange.of(token) as u16;

        let code = format
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
