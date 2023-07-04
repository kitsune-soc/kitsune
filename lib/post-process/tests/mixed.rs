use pest::Parser;
use post_process::{PostParser, Rule};
use std::fs;

#[test]
fn mixed_content() {
    insta::glob!("input/mixed/mixed_content_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::post, &post).unwrap());
    });
}
