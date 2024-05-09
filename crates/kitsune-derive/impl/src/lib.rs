mod expand;

#[proc_macro_attribute]
pub fn kitsune_service(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match self::expand::expand(attrs.into(), input.into()) {
        Ok(out) => out.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
