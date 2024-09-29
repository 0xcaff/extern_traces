use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitInt, Token};

pub enum FromBitsFieldAttribute {
    BitRange(BitRange),
    With(Expr),
}

impl Parse for FromBitsFieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitInt) {
            Ok(FromBitsFieldAttribute::BitRange(input.parse()?))
        } else {
            Ok(FromBitsFieldAttribute::With(input.parse()?))
        }
    }
}

pub struct BitRange {
    pub lowest_bit: u8,
    pub highest_bit: u8,
}

impl Parse for BitRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let highest_bit: LitInt = input.parse()?;
        let _separator: Token![,] = input.parse()?;
        let lowest_bit: LitInt = input.parse()?;

        Ok(Self {
            lowest_bit: lowest_bit.base10_parse()?,
            highest_bit: highest_bit.base10_parse()?,
        })
    }
}
