use macro_utils::exactly_one;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_parse_register_entry(derive_input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let ident = &derive_input.ident;

    let Data::Enum(data) = derive_input.data else {
        return Err(syn::Error::new_spanned(derive_input, "expected enum"));
    };

    let branches = {
        data.variants
            .iter()
            .map(|it| {
                let register_attr = exactly_one(&it.attrs, "register", it)?;

                let ident = &it.ident;

                Ok(quote! {
                    #register_attr => Self::#ident(<_ as FromBits<32>>::from_bits(value)),
                })
            })
            .collect::<Result<Vec<_>, syn::Error>>()?
    };

    Ok(quote! {
        impl crate::registers::entry::ParseRegisterEntry for #ident {
            fn parse_register_entry(register: Register, value: u32) -> Self {
                match register {
                    #(#branches)*

                    _ => panic!("unknown variant {:?}", register)
                }
            }
        }
    })
}
