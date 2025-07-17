use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

struct Attributes {
    other_type: syn::Path,
}

impl Attributes {
    fn from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut other_type = None;

        for attr in attrs {
            if !attr.path().is_ident("two_way_from") {
                continue;
            }

            other_type = Some(attr.parse_args()?);
        }

        Ok(Self {
            other_type: other_type.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "missing attribute \"two_way_from\"")
            })?,
        })
    }
}

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let derive_input = syn::parse2::<DeriveInput>(input)?;
    let attrs = Attributes::from_attrs(&derive_input.attrs)?;

    let name = &derive_input.ident;
    let other_type = &attrs.other_type;

    let syn::Data::Enum(enum_data) = derive_input.data else {
        return Err(syn::Error::new_spanned(derive_input, "expected enum"));
    };

    let variants: Vec<_> = enum_data
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .collect();

    let code = quote! {
        impl From<#other_type> for #name {
            fn from(val: #other_type) -> Self {
                match val {
                    #(
                        #other_type :: #variants => #name :: #variants,
                    )*
                }
            }
        }

        impl From<#name> for #other_type {
            fn from(val: #name) -> Self {
                match val {
                    #(
                        #name :: #variants => #other_type :: #variants,
                    )*
                }
            }
        }
    };
    Ok(code)
}
