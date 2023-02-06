use pest::Parser;
use post_process::{PostParser, Rule};
use std::fs;

#[test]
fn link_only() {
    insta::glob!("input/link/only_link_*", |path| {
        let link = fs::read_to_string(path).unwrap();
        insta::assert_debug_snapshot!(PostParser::parse(Rule::link, &link).unwrap());
    });
}
