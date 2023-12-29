use crate::bitstring::BitString;
use macro_utils::exactly_one;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use std::str::FromStr;
use syn::__private::TokenStream2;
use syn::{parse2, Data, DataEnum, DeriveInput, Field, Type, Variant};

pub fn derive_parse_instruction(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let enum_ident = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "only enums are supported",
        ));
    };

    let variants = get_variant_info(&data_enum)?;
    let blocks = emit_blocks(enum_ident, variants);

    Ok(quote! {
        impl <R: crate::instructions::formats::Reader> crate::instructions::formats::ParseInstruction<R> for #enum_ident {
            fn parse(token: u32, reader: R) -> std::result::Result<Self, crate::instructions::InstructionParseErrorKind> {
                fn bitmask(size: u8) -> u32 {
                    ((1 << size) - 1) << (32 - size)
                }

                fn extract_pattern(token: u32, size: u8) -> u32 {
                    (token & bitmask(size)) >> (32 - size)
                }

                #(#blocks)*

                return Err(::anyhow::format_err!("unable to find matching format for {}", token).into())
            }
        }
    })
}

fn emit_blocks<'a>(
    enum_ident: &'a Ident,
    variant_info: Vec<VariantInfo<'a>>,
) -> impl Iterator<Item = TokenStream> + 'a {
    let groups = {
        let mut groups = BTreeMap::new();

        for info in variant_info {
            groups
                .entry(info.bits.len())
                .or_insert_with(Vec::new)
                .push((info.typ, info.ident, info.bits))
        }

        groups
    };

    groups.into_iter().rev().map(move |(width, pattern)| {
        let width = width as u8;

        let branches = pattern.iter().map(|(typ, ident, bits)| {
            let repr = Literal::from_str(&bits.repr()).unwrap();

            quote! {
                #repr => return Ok(
                    #enum_ident::#ident(
                        <#typ as crate::instructions::formats::ParseInstruction<R>>::parse(token, reader)?
                    )
                ),
            }
        });

        quote! {
            {
                let masked = extract_pattern(token, #width);

                match masked {
                    #(#branches)*
                    _ => {}
                }
            }
        }
    })
}

fn get_variant_info(data_enum: &DataEnum) -> Result<Vec<VariantInfo>, syn::Error> {
    Ok(data_enum
        .variants
        .iter()
        .map(|it| VariantInfo::from(it))
        .collect::<Result<Vec<_>, _>>()?)
}

struct VariantInfo<'a> {
    bits: BitString,
    typ: Type,
    ident: &'a Ident,
}

impl VariantInfo<'_> {
    pub fn from(variant: &Variant) -> Result<VariantInfo, syn::Error> {
        Ok(VariantInfo {
            ident: &variant.ident,
            bits: {
                let pattern_expr = exactly_one(&variant.attrs, "pattern", variant)?;
                parse2(pattern_expr)?
            },
            typ: exactly_one_field(variant)?.ty.clone(),
        })
    }
}

fn exactly_one_field(variant: &Variant) -> Result<&Field, syn::Error> {
    let mut it = variant.fields.iter();

    let field = it
        .next()
        .ok_or(syn::Error::new_spanned(variant, "missing field"))?;

    if let Some(next_field) = it.next() {
        return Err(syn::Error::new_spanned(
            next_field,
            "more than one field in enum variant, expected exactly one",
        ));
    }

    Ok(field)
}
