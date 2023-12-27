mod format_info;
mod generated;
mod operands;

use anyhow::format_err;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::io;
use std::io::{ErrorKind, Read};

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

#[derive(Debug)]
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

    pub fn decode(&mut self) -> Result<Instruction, InstructionParseErrorKind> {
        let token = self.reader.read_u32::<LittleEndian>().wrapping_eof()?;

        let format = self
            .op_format_matcher
            .from(token)
            .ok_or_else(|| format_err!("unknown token {:x}", token))?;

        let format_info = format.info();

        let remaining_len = format_info.dword_len - 1;

        let next_token = if remaining_len == 1 {
            Some(self.reader.read_u32::<LittleEndian>().wrapping_eof()?)
        } else {
            None
        };

        match format {
            OpFormat::VOP3 => {
                for format in [
                    OpFormat::VOP3,
                    OpFormat::VOPC,
                    OpFormat::VOP2,
                    OpFormat::VOP1,
                ] {
                    let op_bits = format.info().op_bits.as_ref().unwrap().of(token) as u16;
                    let code = match format.op(self.level, op_bits) {
                        Some(code) => code,
                        None => continue,
                    };

                    return Ok(Instruction { code });
                }

                return Err(format_err!("no opcode found").into());
            }
            OpFormat::EXP => return Ok(Instruction { code: OpCode::exp }),
            _ => {
                let bitrange = format_info
                    .op_bits
                    .as_ref()
                    .ok_or_else(|| format_err!("unknown op_bits for format {:?}", format))?;

                let op_bits = bitrange.of(token) as u16;

                let code = format
                    .op(self.level, op_bits)
                    .ok_or_else(|| format_err!("no opcode found"))?;

                Ok(Instruction { code })
            }
        }
    }
}

pub enum InstructionParseErrorKind {
    Eof,
    Error(anyhow::Error),
}

trait ResultExt<T> {
    fn wrapping_eof(self) -> Result<T, InstructionParseErrorKind>;
}

impl<T> ResultExt<T> for Result<T, io::Error> {
    fn wrapping_eof(self) -> Result<T, InstructionParseErrorKind> {
        self.map_err(|err| match err.kind() {
            ErrorKind::UnexpectedEof => InstructionParseErrorKind::Eof,
            _ => InstructionParseErrorKind::Error(err.into()),
        })
    }
}

impl<T> From<T> for InstructionParseErrorKind
where
    T: Into<anyhow::Error>,
{
    fn from(value: T) -> Self {
        InstructionParseErrorKind::Error(value.into())
    }
}
