use pest::Parser;
use post_process::{PostParser, Rule};
use std::fs;

#[test]
fn parse_emote() {
    insta::glob!("input/emote/full_post_*", |path| {
        let post = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::post, &post).unwrap());
    });
}
