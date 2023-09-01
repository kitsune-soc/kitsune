use crate::util::parse_to_test_output;
use std::fs;

mod util;

#[test]
fn invalid_mention() {
    insta::glob!("input/mention/invalid_*", |path| {
        let mention = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&mention));
    });
}

#[test]
fn subdomain_mention() {
    insta::glob!("input/mention/subdomain_*", |path| {
        let mention = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&mention));
    })
}

#[test]
fn parse_mention() {
    insta::glob!("input/mention/full_post_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(parse_to_test_output(&post));
    });
}
