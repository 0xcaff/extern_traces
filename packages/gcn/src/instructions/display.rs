pub trait DisplayInstruction {
    fn display(&self, literal_constant: Option<u32>) -> DisplayableInstruction;
}

#[derive(Debug)]
pub struct DisplayableInstruction {
    pub op: String,
    pub args: Vec<String>,
}

impl DisplayableInstruction {
    pub fn to_string(&self) -> String {
        format!("{} {}", self.op, self.args.join(", "))
    }
}
