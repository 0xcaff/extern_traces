mod generated;
mod instruction;

impl OpCode {
    pub fn op_info(&self) -> &'static OpInfo {
        OPS[self]
    }
}

impl OpFormat {
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
}

pub use generated::*;
