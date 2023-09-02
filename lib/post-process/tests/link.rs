use crate::util::parse_to_test_output;
use std::fs;

mod util;

#[test]
fn link_only() {
    insta::glob!("input/link/only_link_*", |path| {
        let link = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&link));
    });
}
