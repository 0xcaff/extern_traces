pub trait DisplayInstruction {
    fn display(&self) -> DisplayableInstruction;
}

#[derive(Debug)]
pub struct DisplayableInstruction {
    pub op: String,
    pub args: Vec<String>,
}
