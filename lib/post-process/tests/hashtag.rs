use crate::util::parse_to_test_output;
use std::fs;

mod util;

#[test]
fn invalid_hashtag() {
    insta::glob!("input/hashtag/invalid_*", |path| {
        let hashtag = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&hashtag));
    });
}

#[test]
fn weird_tags() {
    insta::glob!("input/hashtag/weird_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&post));
    });
}

#[test]
fn parse_hashtag() {
    insta::glob!("input/hashtag/full_post_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&post));
    });
}
