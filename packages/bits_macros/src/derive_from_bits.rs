use macro_utils::exactly_one;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Data, DeriveInput, Field, LitInt, Token};

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
            fn from_bits(value: usize) -> Self {
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
    let args: BitsAttributeArgs = parse2(attribute)?;
    let highest_bit = args.highest_bit;
    let lowest_bit = args.lowest_bit;

    let len = (args.highest_bit - args.lowest_bit + 1) as usize;

    Ok(quote! {
        #identifier: <#typ as ::bits::FromBits<#len>>::from_bits(::bits::bitrange(#highest_bit, #lowest_bit).of(value)),
    })
}

struct BitsAttributeArgs {
    lowest_bit: u8,
    highest_bit: u8,
}

impl Parse for BitsAttributeArgs {
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
