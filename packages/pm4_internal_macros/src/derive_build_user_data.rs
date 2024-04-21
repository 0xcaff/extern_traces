use macro_utils::exactly_one;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::ops::Range;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, DeriveInput, LitInt, Token};

struct DeriveUserDataArgs {
    ident_prefix: Ident,
    range: Range<u8>,
}

impl Parse for DeriveUserDataArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident_prefix = input.parse()?;
        let _separator: Token![,] = input.parse()?;

        let range_starting: LitInt = input.parse()?;

        let _another_separator: Token![..=] = input.parse()?;

        let range_ending: LitInt = input.parse()?;

        let end: u8 = range_ending.base10_parse()?;

        Ok(Self {
            ident_prefix,
            range: Range {
                start: range_starting.base10_parse()?,
                end: end + 1,
            },
        })
    }
}

pub fn derive_build_user_data(derive_input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let ident = &derive_input.ident;

    let builder_ident_name = format!("{}Builder", ident);
    let builder_ident = Ident::new(&builder_ident_name, Span::call_site());

    let args = exactly_one(&derive_input.attrs, "user_data", &derive_input)?;
    let args: DeriveUserDataArgs = parse2(args)?;

    let branch_arms = args.range.into_iter().map(|it| {
        let ident_name = format!("{}{}", args.ident_prefix, it);
        let ident = Ident::new(&ident_name, Span::call_site());

        quote! {
            RegisterEntry::#ident(value) => { self.entries.insert(#it, *value); },
        }
    });

    let update_block = quote! {
        impl crate::intermediate::build::Builder<RegisterEntry> for #builder_ident {
            fn update(&mut self, entry: &RegisterEntry) -> Option<()> {
                match entry {
                    #(#branch_arms)*
                    _ => {
                        return None
                    }
                }

                Some(())
            }
        }
    };

    Ok(quote! {
        impl crate::intermediate::build::BuildBase for #ident {
            type Builder = #builder_ident;
        }

        impl crate::intermediate::build::Build<RegisterEntry> for #ident {
            type Builder = #builder_ident;
        }

        #[derive(Clone)]
        pub struct #builder_ident {
            entries: BTreeMap<u8, u32>,
        }

        impl crate::intermediate::build::Initialize for #builder_ident {
            fn new() -> Self {
                Self {
                    entries: BTreeMap::new(),
                }
            }
        }

        #update_block

        impl crate::intermediate::build::Finalize<#ident> for #builder_ident {
            fn finalize(self) -> Result<#ident, anyhow::Error> {
                Ok(#ident(self.entries))
            }
        }
    })
}
