use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse_quote,
    visit_mut::{visit_expr_mut, visit_file_mut, VisitMut},
    Expr, ImplItemFn, ItemFn, TraitItemFn,
};

struct TryBlockVisitor {
    use_async_strategy: bool,
}

impl VisitMut for TryBlockVisitor {
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        let Expr::TryBlock(ref try_block) = expr else {
            visit_expr_mut(self, expr);
            return;
        };

        let body = &try_block.block;
        let replacement: Expr = if self.use_async_strategy {
            parse_quote!((async { Ok(#body) }).await)
        } else {
            parse_quote!((|| { Ok(#body) })())
        };

        *expr = replacement;
        visit_expr_mut(self, expr);
    }
}

macro_rules! impl_visit {
    ($($fn_name:ident => $ty:ty),+$(,)?) => {
        $(
            fn $fn_name(&mut self, func: &mut $ty) {
                let use_async_strategy = func.sig.asyncness.is_some();
                let mut visitor = TryBlockVisitor { use_async_strategy };
                ::syn::visit_mut::$fn_name(&mut visitor, func);
            }
        )+
    };
}

struct FunctionVisitor;

impl VisitMut for FunctionVisitor {
    impl_visit! {
        visit_impl_item_fn_mut => ImplItemFn,
        visit_item_fn_mut => ItemFn,
        visit_trait_item_fn_mut => TraitItemFn,
    }
}

fn trials_impl(input: TokenStream) -> TokenStream {
    let mut code: syn::File = match syn::parse2(input) {
        Ok(func) => func,
        Err(err) => return err.into_compile_error(),
    };

    let mut visitor = FunctionVisitor;
    visit_file_mut(&mut visitor, &mut code);

    code.to_token_stream()
}

/// Usage:
///
/// ```
/// # use trials::trials_stable;
/// trials_stable! {
///     pub fn fallible() {
///         let result: Result<_, ()> = try {
///             # let fallible_operation = move || Err::<(), ()>(());
///             fallible_operation()?;
///             Ok(())
///         };
///
///         assert!(result.is_err());
///     }
/// }
#[proc_macro]
pub fn trials_stable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    trials_impl(input.into()).into()
}

/// ## ⚠ HUGE DISCLAIMER
///
/// Since this is an attribute macro, it still goes through the Rust parser, meaning if (for some reason) `try` blocks
/// are fully removed from the Rust parser, compilation will start failing.
///
/// Only use this if you really don't care about your code potentially breaking (and about the large warning block rustc emits)
///
/// ---
///
/// Usage:
///
/// ```
/// # use trials::trials;
/// #[trials]
/// pub fn fallible() {
///     let result: Result<_, ()> = try {
///         # let fallible_operation = move || Err::<(), ()>(());
///         fallible_operation()?;
///         Ok(())
///     };
///
///     assert!(result.is_err());
/// }
/// ```
#[proc_macro_attribute]
pub fn trials(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attr.is_empty() {
        return syn::Error::new_spanned(TokenStream::from(attr), "expected no arguments")
            .into_compile_error()
            .into();
    }

    trials_impl(input.into()).into()
}
