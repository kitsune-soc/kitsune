mod service;
mod two_way_from;

#[proc_macro_attribute]
pub fn kitsune_service(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match self::service::expand(attrs.into(), input.into()) {
        Ok(out) => out.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(TwoWayFrom, attributes(two_way_from))]
pub fn two_way_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match self::two_way_from::expand(input.into()) {
        Ok(out) => out.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
