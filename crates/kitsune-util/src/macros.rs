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

        $crate::tokio::try_join!(
            $( assert_send($try_future) ),+
        )
    }};
}
