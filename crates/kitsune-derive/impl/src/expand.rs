use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::iter;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Token, Visibility,
};

struct Attributes {
    expand_builder: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            expand_builder: true,
        }
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut attributes = Self::default();
        let punctuated = Punctuated::<syn::Ident, Token![,]>::parse_terminated(input)?;

        for ident in punctuated {
            if ident == "omit_builder" {
                attributes.expand_builder = false;
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unknown attribute: {ident}"),
                ));
            }
        }

        Ok(attributes)
    }
}

fn expand_builder(
    parsed_struct: &syn::ItemStruct,
    inner_struct_name: &syn::Ident,
) -> (TokenStream, TokenStream) {
    let struct_name = &parsed_struct.ident;
    let inner_builder_name = format_ident!("{inner_struct_name}Builder");

    let num_lifetimes = parsed_struct.generics.lifetimes().count();
    let lifetimes = iter::repeat(quote!('_)).take(num_lifetimes);

    let attrs = quote! {
        #[derive(::kitsune_derive::typed_builder::TypedBuilder)]
        #[builder(build_method(into = #struct_name))]
        #[builder(crate_module_path = ::kitsune_derive::typed_builder)]
    };
    let impls = quote! {
        impl #struct_name {
            pub fn builder() -> #inner_builder_name<#(#lifetimes),*> {
                #inner_struct_name::builder()
            }
        }
    };

    (attrs, impls)
}

pub fn expand(attrs: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let attributes = syn::parse2::<Attributes>(attrs)?;
    let mut parsed_struct = syn::parse2::<syn::ItemStruct>(input)?;

    let struct_name = parsed_struct.ident.clone();
    let inner_struct_name = format_ident!("__{struct_name}__Inner");

    let (builder_attrs, builder_impls) = if attributes.expand_builder {
        expand_builder(&parsed_struct, &inner_struct_name)
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    parsed_struct.ident = inner_struct_name.clone();
    parsed_struct.vis = Visibility::Public(Token![pub](parsed_struct.span()));

    let output = quote! {
        #builder_attrs
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #parsed_struct

        #[derive(Clone)]
        pub struct #struct_name {
            inner: ::kitsune_derive::triomphe::Arc<#inner_struct_name>,
        }

        #builder_impls

        impl ::core::ops::Deref for #struct_name {
            type Target = #inner_struct_name;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl From<#inner_struct_name> for #struct_name {
            fn from(inner: #inner_struct_name) -> Self {
                Self {
                    inner: ::kitsune_derive::triomphe::Arc::new(inner),
                }
            }
        }
    };

    Ok(output)
}
