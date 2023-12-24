use crate::instructions::{OpFormat, OpInfo, OPS};
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
        op_bits: None,
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

pub struct OpFormatInfo {
    pub format: OpFormat,
    pub bytes_len: u8,

    pub pattern: Option<OpFormatPattern>,
    pub op_bits: Option<BitRange>,
}

pub struct OpFormatPattern {
    pub width: u8,
    pub bits: u32,
}

pub struct BitRange {
    pub start: u8,
    pub len: u8,
}

impl BitRange {
    pub fn of(&self, value: u32) -> u32 {
        let offset = 32 - self.len - self.start;

        let mask = ((1 << self.len) - 1) << offset;

        (mask & value) >> offset
    }
}

pub const fn bitrange(start: u8, len: u8) -> BitRange {
    BitRange { start, len }
}


impl OpFormat {
    pub fn ops(&self) -> impl Iterator<Item = &'static OpInfo> + '_ {
        OPS.iter().filter(move |it| &it.format == self)
    }

    pub fn info(&self) -> &'static OpFormatInfo {
        &OP_FORMATS[(*self) as usize]
    }
}
