use proc_macro2::TokenStream;
use quote::ToTokens;

use syn::{Attribute, Meta};

pub fn upto_one(attrs: &[Attribute], name: &str) -> Result<Option<TokenStream>, syn::Error> {
    let mut filtered_attributes = attrs.iter().filter_map(|attr| {
        let Meta::List(value) = &attr.meta else {
            return None;
        };

        if !value.path.is_ident(name) {
            return None;
        }

        Some(value.tokens.clone())
    });

    let value = filtered_attributes.next();
    match value {
        Some(it) => {
            if let Some(unexpected) = filtered_attributes.next() {
                return Err(syn::Error::new_spanned(
                    unexpected,
                    format!("multiple #[{}(...)]", name),
                ));
            };

            Ok(Some(it))
        }
        None => return Ok(None),
    }
}

pub fn exactly_one(
    attrs: &[Attribute],
    name: &str,
    error_span: impl ToTokens,
) -> Result<TokenStream, syn::Error> {
    let value = upto_one(attrs, name)?;

    let Some(expr) = value else {
        return Err(syn::Error::new_spanned(
            error_span,
            format!("missing #[{}(...)]", name),
        ));
    };

    Ok(expr)
}
