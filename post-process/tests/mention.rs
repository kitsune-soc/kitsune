use pest::Parser;
use post_process::{PostParser, Rule};
use std::fs;

#[test]
fn invalid_mention() {
    insta::glob!("input/mention/invalid_*", |path| {
        let mention = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::post, &mention).unwrap());
    });
}

#[test]
fn subdomain_mention() {
    insta::glob!("input/mention/subdomain_*", |path| {
        let mention = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::mention, &mention).unwrap());
    })
}

#[test]
fn parse_mention() {
    insta::glob!("input/mention/full_post_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::post, &post).unwrap());
    });
}
