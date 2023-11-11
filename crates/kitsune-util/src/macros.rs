/// Implement the `From` trait for each branch of an enum
///
/// ```
/// # use kitsune_util::impl_from;
/// impl_from! {
///     #[derive(Debug, PartialEq)]
///     enum Test {
///         A(i32),
///         B(u32),
///     }
/// }
///
/// assert_eq!(Test::from(1_i32), Test::A(1));
/// assert_eq!(Test::from(2_u32), Test::B(2));
/// ```
#[macro_export]
macro_rules! impl_from {
    (
        $(#[$top_annotation:meta])*
        $vb:vis enum $name:ident {
        $(
            $(#[$branch_annotation:meta])*
            $branch_name:ident ($from_type:ty)
        ),+
        $(,)*
    }) => {
        $(#[$top_annotation])*
        $vb enum $name {
            $(
                $(#[$branch_annotation])*
                $branch_name($from_type),
            )*
        }

        $(
            impl From<$from_type> for $name {
                fn from(val: $from_type) -> Self {
                    Self::$branch_name(val)
                }
            }
        )*
    };
}

/// Wrapper around the [`tokio::try_join`] macro but it passes each future through
/// a small no-op function that gives the compiler some trait bound hints
#[macro_export]
macro_rules! try_join {
    ($($try_future:expr),+$(,)?) => {{
        /// Hack around the [bogus "higher-ranked lifetime" errors](https://github.com/rust-lang/rust/issues/102211)
        ///
        /// Asserts `Send` bounds via its type signature and helps the compiler a little bit with proving the bound
        #[inline(always)]
        fn assert_send<O>(
            fut: impl ::core::future::Future<Output = O> + Send,
        ) -> impl ::core::future::Future<Output = O> + Send {
            fut
        }

        ::tokio::try_join!(
            $( assert_send($try_future) ),+
        )
    }};
}
