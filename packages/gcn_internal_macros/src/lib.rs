mod bitstring;
mod derive_display_instruction;
mod derive_parse_instruction;

use crate::derive_display_instruction::derive_display_instruction;
use crate::derive_parse_instruction::derive_parse_instruction;
use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(ParseInstruction, attributes(pattern))]
pub fn derive_parse_instruction_exported(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    TokenStream::from(derive_parse_instruction(input).unwrap_or_else(|err| err.to_compile_error()))
}

#[proc_macro_derive(DisplayInstruction)]
pub fn derive_display_instruction_exported(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    TokenStream::from(
        derive_display_instruction(input).unwrap_or_else(|err| err.to_compile_error()),
    )
}
