use crate::instructions::formats::ds::DSInstruction;
use crate::instructions::formats::exp::ExpInstruction;
use crate::instructions::formats::mimg::MIMGInstruction;
use crate::instructions::formats::mtbuf::MTBufInstruction;
use crate::instructions::formats::mubuf::MUBUFInstruction;
use crate::instructions::formats::smem::SMEMInstruction;
use crate::instructions::formats::sop1::SOP1Instruction;
use crate::instructions::formats::sop2::SOP2Instruction;
use crate::instructions::formats::sopc::SOPCInstruction;
use crate::instructions::formats::sopk::SOPKInstruction;
use crate::instructions::formats::sopp::SOPPInstruction;
use crate::instructions::formats::vintrp::VINTRPInstruction;
use crate::instructions::formats::vop1::VOP1Instruction;
use crate::instructions::formats::vop2::VOP2Instruction;
use crate::instructions::formats::vop3::VOP3Instruction;
use crate::instructions::formats::vopc::VOPCInstruction;
use crate::instructions::InstructionParseErrorKind;

mod ds;
mod exp;
mod mimg;
mod mtbuf;
mod mubuf;
mod smem;
mod sop1;
mod sop2;
mod sopc;
mod sopk;
mod sopp;
mod vintrp;
mod vop1;
mod vop2;
mod vop3;
mod vopc;

// #[derive(ParseInstruction)]
pub enum FormattedInstruction {
    #[pattern("110110")]
    DS(DSInstruction),

    #[pattern("111110")]
    EXP(ExpInstruction),

    #[pattern("111100")]
    MIMG(MIMGInstruction),

    #[pattern("111010")]
    MTBUF(MTBufInstruction),

    #[pattern("111000")]
    MUBUF(MUBUFInstruction),

    #[pattern("11000")]
    SMEM(SMEMInstruction),

    #[pattern("101111101")]
    SOP1(SOP1Instruction),

    #[pattern("10")]
    SOP2(SOP2Instruction),

    #[pattern("101111100")]
    SOPC(SOPCInstruction),

    #[pattern("1011")]
    SOPK(SOPKInstruction),

    #[pattern("101111111")]
    SOPP(SOPPInstruction),

    #[pattern("110010")]
    VINTRP(VINTRPInstruction),

    #[pattern("0111111")]
    VOP1(VOP1Instruction),

    #[pattern("0")]
    VOP2(VOP2Instruction),

    #[pattern("110100")]
    VOP3(VOP3Instruction),

    #[pattern("0111110")]
    VOPC(VOPCInstruction),
}

pub trait Reader {
    fn read_u32(&mut self) -> Result<u32, InstructionParseErrorKind>;
}

pub trait ParseInstruction<R: Reader> {
    fn parse(token: u32, reader: R) -> Result<Self, InstructionParseErrorKind>;
}
