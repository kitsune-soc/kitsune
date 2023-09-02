use crate::util::parse_to_test_output;
use std::fs;

mod util;

#[test]
fn parse_emote() {
    insta::glob!("input/emote/full_post_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&post));
    });
}
