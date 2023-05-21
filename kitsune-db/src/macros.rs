#[macro_export]
#[doc(hidden)]
macro_rules! impl_columns {
    ($struct:ty => $($columns:tt)*) => {
        impl $struct {
            #[allow(clippy::type_complexity)]
            pub const fn columns() -> $($columns)* {
                $($columns)*
            }
        }
    };
}
