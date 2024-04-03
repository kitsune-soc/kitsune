#[cfg(feature = "proc-macro")]
pub use trials_macros::{trials, trials_stable};

#[macro_export]
macro_rules! attempt {
    (async $($tt:tt)*) => {
        async { Ok({ $($tt)* }) }.await
    };
    ($($tt:tt)*) => {
        (|| Ok({ $($tt)* }))()
    };
}
