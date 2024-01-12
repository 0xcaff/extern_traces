use crate::Register;

pub trait ParseRegisterEntry {
    fn parse_register_entry(register: Register, value: u32) -> Self;
}
