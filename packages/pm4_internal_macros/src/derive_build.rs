use macro_utils::{exactly_one, upto_one};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Type};

pub fn derive_build(derive_input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let ident = &derive_input.ident;

    let Data::Struct(data) = &derive_input.data else {
        return Err(syn::Error::new_spanned(derive_input, "expected struct"));
    };

    let entry_ident = exactly_one(&derive_input.attrs, "entry", &derive_input)?;

    let builder_ident_name = format!("{}Builder", ident);
    let builder_ident = Ident::new(&builder_ident_name, Span::call_site());

    struct Field {
        ident: Ident,
        ty: Type,
        attribute: Option<TokenStream>,
    }

    let fields = data
        .fields
        .iter()
        .map(|it| {
            let ident = it.ident.as_ref().ok_or_else(|| {
                syn::Error::new(it.span(), "anonymous struct handling unimplemented")
            })?;
            let ty = &it.ty;

            let attribute = upto_one(&it.attrs, "entry")?;

            Ok(Field {
                ident: ident.clone(),
                ty: ty.clone(),
                attribute,
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let builder_fields = fields.iter().map(|it| {
        let ident = &it.ident;
        let ty = &it.ty;

        if let Some(_attribute) = &it.attribute {
            quote! {
                #ident: <#ty as crate::intermediate::build::BuildBase>::Builder,
            }
        } else {
            quote! {
                #ident: <#ty as crate::intermediate::build::Build<#entry_ident>>::Builder,
            }
        }
    });

    let initializer_fields = fields.iter().map(|it| {
        let ident = &it.ident;
        let ty = &it.ty;

        if let Some(_attribute) = &it.attribute {
            quote! {
                #ident: <#ty as crate::intermediate::build::BuildBase>::Builder::new(),
            }
        } else {
            quote! {
                #ident: <#ty as crate::intermediate::build::Build<#entry_ident>>::Builder::new(),
            }
        }
    });

    let init_impl = quote! {
        impl crate::intermediate::build::Initialize for #builder_ident {
            fn new() -> Self {
                Self {
                    #(#initializer_fields)*
                }
            }
        }
    };

    let (match_arms, fallback_statements) = {
        let mut match_arms = vec![];
        let mut fallback_statements = vec![];

        for it in &fields {
            let ident = &it.ident;

            if let Some(attribute) = &it.attribute {
                let exp = quote! {
                    #attribute(value) => {
                        self.#ident = Some((*value).clone().into());
                        return Some(())
                    },
                };
                match_arms.push(exp);
            } else {
                let stmt = quote! {
                    if let Some(()) = self.#ident.update(entry) {
                        return Some(())
                    }
                };

                fallback_statements.push(stmt);
            }
        }

        (match_arms, fallback_statements)
    };

    let update_impl = quote! {
        impl crate::intermediate::build::Builder<#entry_ident> for #builder_ident {
            fn update(&mut self, entry: &#entry_ident) -> Option<()> {
                match entry {
                    #(#match_arms)*
                    _ => {
                        #(#fallback_statements)*
                    }
                }

                None
            }
        }
    };

    let finalize_expressions = fields
        .iter()
        .map(|it| {
            let ident = &it.ident;

            Ok(quote! {
                #ident: self.#ident.finalize().with_context(|| format!("for field {}", stringify!(#ident)))?,
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let finalize_impl = quote! {
        impl crate::intermediate::build::Finalize<#ident> for #builder_ident {
            fn finalize(self) -> Result<#ident, ::anyhow::Error> {
                use anyhow::Context;

                Ok(#ident {
                    #(#finalize_expressions)*
                })
            }
        }
    };

    Ok(quote! {
        #[derive(Clone)]
        pub struct #builder_ident {
            #(#builder_fields)*
        }

        impl crate::intermediate::build::Build<#entry_ident> for #ident {
            type Builder = #builder_ident;
        }

        impl crate::intermediate::build::Marker for #ident {
        }

        #init_impl
        #update_impl
        #finalize_impl
    })
}
