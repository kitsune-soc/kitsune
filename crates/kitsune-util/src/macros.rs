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
