pub trait ParseRegisterEntry {
    fn parse_register_entry(register: u32, value: u32) -> Self;
}
