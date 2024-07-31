#[macro_export]
macro_rules! attempt {
    (async $($tt:tt)*) => {
        async { Ok({ $($tt)* }) }.await
    };
    ($($tt:tt)*) => {
        (|| Ok({ $($tt)* }))()
    };
}
