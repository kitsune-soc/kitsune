use crate::util::parse_to_test_output;
use std::fs;

mod util;

#[test]
fn mixed_content() {
    insta::glob!("input/mixed/mixed_content_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&post));
    });
}
