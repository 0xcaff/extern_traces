mod derive_from_bits;

use crate::derive_from_bits::derive_from_bits;
use proc_macro::TokenStream;
use syn::DeriveInput;

/// # Example
///
/// ```
/// use bits_macros::FromBits;
///
/// #[derive(FromBits)]
/// #[bits(64)]
/// struct Item {
///     #[bits(31, 0)]
///     field1: u32,
///
///     #[bits(40, 32)]
///     field2: u8,
/// }
/// ```
///
/// The struct attribute `#[bits(N)]` specifies the BITS const argument of the `FromBits` trait.
/// The field attributes `#[bits(highest_bit, lowest_bit)]` specifies the inclusive range of bits
/// which correspond to the annotated field. Bit 0 is interpreted as the least significant bit and
/// bit N, the most significant bit. Both bounds are inclusive.
#[proc_macro_derive(FromBits, attributes(bits))]
pub fn derive_from_bits_exported(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    TokenStream::from(derive_from_bits(input).unwrap_or_else(|err| err.to_compile_error()))
}
