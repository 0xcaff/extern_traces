pub struct InstructionInfo {
    pub definitions: &'static [Option<OperandInfo>; 4],
    pub operands: &'static [Option<OperandInfo>; 4],
}

pub enum OperandInfo {
    M0,
    SCC,
    Exec,
    ExecLo,
    Vcc,
    Size(u8),
}
