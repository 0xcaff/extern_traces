use syn::parse::{Parse, ParseStream};
use syn::LitInt;

/// Holds a 0b integer literal. Preserves leading 0s.
pub struct BitString {
    // Ordered from most significant byte to least significant byte. Includes leading zeros.
    value: Vec<bool>,
}

impl Parse for BitString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<LitInt>()?;

        // Expecting the repr here to be mostly stable as it should be the same
        // as expressed in input source code.
        let repr = lit.to_string();

        if !repr.starts_with("0b") {
            return Err(input.error("expected a binary literal starting with 0b"));
        }

        let mut result = Vec::new();

        let bits_repr = &repr[2..];
        for bit in bits_repr.chars() {
            match bit {
                '0' => result.push(false),
                '1' => result.push(true),
                _ => return Err(input.error(format!("unexpected value {}", bit))),
            }
        }

        Ok(BitString { value: result })
    }
}

impl BitString {
    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn repr(&self) -> String {
        let mut result = "0b".to_string();

        for value in self.value.iter() {
            result.push_str(if *value { "1" } else { "0" })
        }

        result
    }
}
