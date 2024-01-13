mod derive_build;
mod derive_parse_register_entry;

use crate::derive_build::derive_build;
use crate::derive_parse_register_entry::derive_parse_register_entry;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(ParseRegisterEntry, attributes(register))]
pub fn derive_parse_register_entry_exported(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    TokenStream::from(
        derive_parse_register_entry(input).unwrap_or_else(|err| err.to_compile_error()),
    )
}

#[proc_macro_derive(Build, attributes(entry))]
pub fn derive_build_exported(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    TokenStream::from(derive_build(input).unwrap_or_else(|err| err.to_compile_error()))
}
