use macro_utils::exactly_one_field;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_parse_packet_value(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let ident = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        return Err(syn::Error::new_spanned(&input, "only enums are supported"));
    };

    let branches = data_enum
        .variants
        .iter()
        .filter(|it| {
            it.ident.to_string() != "Unknown"
        })
        .map(|it| -> Result<_, syn::Error> {
            let field = exactly_one_field(&it.fields)?;
            let variant_ident = &it.ident;

            let variant_type = &field.ty;

            Ok(quote! {
                <#variant_type as crate::packet_value::ParseType3Packet>::CODE =>
                    #ident::#variant_ident(<#variant_type as crate::packet_value::ParseType3Packet>::parse_type3_packet(body)),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        impl ParsePacketValue for #ident {
            fn parse(op: crate::op_codes::OpCode, body: Vec<u32>) -> Self {
                use crate::packet_value::ParseType3Packet;

                match op {
                    #(#branches)*
                    _ => Self::Unknown { op, body }
                }
            }
        }
    })
}
