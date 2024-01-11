mod derive_from_bits;

use crate::derive_from_bits::derive_from_bits;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(FromBits, attributes(bits))]
pub fn derive_from_bits_exported(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    TokenStream::from(derive_from_bits(input).unwrap_or_else(|err| err.to_compile_error()))
}
