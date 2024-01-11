use macro_utils::exactly_one;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Data, DeriveInput, Field, LitInt, Token};

pub fn derive_from_bits(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_ident = &input.ident;

    let attribute = exactly_one(&input.attrs, "bits", &input)?;
    let bits_length = parse2::<LitInt>(attribute)?.base10_parse()?;

    let Data::Struct(struct_value) = input.data else {
        return Err(syn::Error::new_spanned(input, "only struct implemented"));
    };

    let initializers = struct_value
        .fields
        .iter()
        .map(|it| field(it, bits_length))
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

fn field(field: &Field, total_len: usize) -> Result<TokenStream, syn::Error> {
    let identifier = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "only non-anonymous fields allowed"))?;
    let typ = &field.ty;

    let attribute = exactly_one(&field.attrs, "bits", &field)?;
    let args: BitsAttributeArgs = parse2(attribute)?;

    let len = (args.highest_bit - args.lowest_bit + 1) as usize;

    let start = (total_len - args.lowest_bit as usize - len) as u8;

    let len_short = len as u8;

    // todo: remove of_32 hardcode
    if total_len == 32 {
        Ok(quote! {
            #identifier: <#typ as ::bits::FromBits<#len>>::from_bits(::bits::bitrange(#start, #len_short).of_32(value as u32)),
        })
    } else {
        Ok(quote! {
            #identifier: <#typ as ::bits::FromBits<#len>>::from_bits(::bits::bitrange(#start, #len_short).of_64(value as u64)),
        })
    }
}

struct BitsAttributeArgs {
    lowest_bit: u8,
    highest_bit: u8,
}

impl Parse for BitsAttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lowest_bit: LitInt = input.parse()?;
        let _separator: Token![,] = input.parse()?;
        let highest_bit: LitInt = input.parse()?;

        Ok(Self {
            lowest_bit: lowest_bit.base10_parse()?,
            highest_bit: highest_bit.base10_parse()?,
        })
    }
}
