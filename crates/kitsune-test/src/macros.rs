#[macro_export]
macro_rules! assert_display_eq {
    ($left:expr_2021, $right:expr_2021 $(, $msg:literal)?) => {
        assert_eq!(
            $left.to_string(),
            $right.to_string()
            $(, $msg)?
        )
    };
}
