use pest::Parser;
use post_process::{PostParser, Rule};
use std::fs;

#[test]
fn invalid_hashtag() {
    insta::glob!("input/hashtag/invalid_*", |path| {
        let hashtag = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::hashtag, &hashtag));
    });
}

#[test]
fn weird_tags() {
    insta::glob!("input/hashtag/weird_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::post, &post).unwrap());
    });
}

#[test]
fn parse_hashtag() {
    insta::glob!("input/hashtag/full_post_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::post, &post).unwrap());
    });
}
