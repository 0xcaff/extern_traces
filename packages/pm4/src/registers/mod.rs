mod entry;
pub mod generated;
use generated::Register;

pub use entry::ParseRegisterEntry;

impl Register {
    pub fn decode(address: usize) -> Result<Self, anyhow::Error> {
        Ok(Self::from_repr(address)
            .ok_or_else(|| anyhow::format_err!("unknown register {}", address))?)
    }
}
