#[macro_export]
macro_rules! assert_display_eq {
    ($left:expr, $right:expr $(, $msg:literal)?) => {
        assert_eq!(
            $left.to_string(),
            $right.to_string()
            $(, $msg)?
        )
    };
}
