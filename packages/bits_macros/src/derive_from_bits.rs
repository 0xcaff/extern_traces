use crate::attrs::FromBitsFieldAttribute;
use macro_utils::exactly_one;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse2, Data, DeriveInput, Field, LitInt};

pub fn derive_from_bits(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_ident = &input.ident;

    let attribute = exactly_one(&input.attrs, "bits", &input)?;
    let bits_length: usize = parse2::<LitInt>(attribute)?.base10_parse()?;

    let Data::Struct(struct_value) = input.data else {
        return Err(syn::Error::new_spanned(input, "only struct implemented"));
    };

    let initializers = struct_value
        .fields
        .iter()
        .map(|it| field(it))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        impl ::bits::FromBits<#bits_length> for #struct_ident {
            fn from_bits(value: impl ::bits::Bits) -> Self {
                #struct_ident {
                    #(#initializers)*
                }
            }
        }
    })
}

fn field(field: &Field) -> Result<TokenStream, syn::Error> {
    let identifier = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "only non-anonymous fields allowed"))?;
    let typ = &field.ty;

    let attribute = exactly_one(&field.attrs, "bits", &field)?;
    let attribute_span = attribute.span();
    let args: FromBitsFieldAttribute = parse2(attribute)?;

    Ok(match &args {
        FromBitsFieldAttribute::BitRange(range) => {
            let highest_bit = range.highest_bit;
            let lowest_bit = range.lowest_bit;

            if highest_bit < lowest_bit {
                return Err(syn::Error::new(
                    attribute_span,
                    format!(
                        "second value ({}) larger than first value ({})",
                        highest_bit, lowest_bit
                    ),
                ));
            }

            let len = (range.highest_bit - range.lowest_bit + 1) as usize;

            quote! {
                #identifier: <#typ as ::bits::FromBits<#len>>::from_bits(value.slice(#highest_bit, #lowest_bit)),
            }
        }
        FromBitsFieldAttribute::With(expr) => {
            quote! {
                #identifier: #expr(value),
            }
        }
    })
}
