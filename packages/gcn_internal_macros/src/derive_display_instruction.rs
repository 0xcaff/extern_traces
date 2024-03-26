use macro_utils::exactly_one_field;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{Data, DeriveInput};

pub fn derive_display_instruction(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let enum_ident = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        return Err(syn::Error::new_spanned(&input, "only enums are supported"));
    };

    let branches = data_enum
        .variants
        .iter()
        .map(|it| -> Result<_, syn::Error> {
            let _field = exactly_one_field(&it.fields)?;

            let variant_ident = &it.ident;

            Ok(quote! {
                #enum_ident::#variant_ident(it) => crate::DisplayInstruction::display(it, literal_constant),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        impl crate::DisplayInstruction for FormattedInstruction {
            fn display(&self, literal_constant: Option<u32>) -> crate::DisplayableInstruction {
                match self {
                    #(#branches)*
                }
            }
        }
    })
}
