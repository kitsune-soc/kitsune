use pest::Parser;
use post_process::{PostParser, Rule};
use pretty_assertions::assert_eq;

#[test]
fn invalid_hashtag() {
    let text = "##invalid";
    assert!(PostParser::parse(Rule::hashtag, text).is_err());
}

#[test]
fn parse_hashtag() {
    let text = "why am i building a #lexer for #posts? #龍が如く0";
    let mut token_iter = PostParser::parse(Rule::post, text).expect("Failed to parse post");

    let text1 = token_iter.next().unwrap();
    assert_eq!(text1.as_rule(), Rule::text);
    assert_eq!(text1.as_str(), "why am i building a");

    let hashtag1 = token_iter.next().unwrap();
    assert_eq!(hashtag1.as_rule(), Rule::hashtag);
    assert_eq!(hashtag1.as_str(), " #lexer");

    let mut hashtag1 = hashtag1.into_inner();
    let hashtag1_prefix = hashtag1.next().unwrap();
    let hashtag1_content = hashtag1.next().unwrap();

    assert_eq!(hashtag1_prefix.as_rule(), Rule::component_prefix);
    assert_eq!(hashtag1_prefix.as_str(), " ");
    assert_eq!(hashtag1_content.as_rule(), Rule::hashtag_content);
    assert_eq!(hashtag1_content.as_str(), "lexer");

    let text2 = token_iter.next().unwrap();
    assert_eq!(text2.as_rule(), Rule::text);
    assert_eq!(text2.as_str(), " for");

    let hashtag2 = token_iter.next().unwrap();
    assert_eq!(hashtag2.as_rule(), Rule::hashtag);
    assert_eq!(hashtag2.as_str(), " #posts");

    let mut hashtag2 = hashtag2.into_inner();
    let hashtag2_prefix = hashtag2.next().unwrap();
    let hashtag2_content = hashtag2.next().unwrap();

    assert_eq!(hashtag2_prefix.as_rule(), Rule::component_prefix);
    assert_eq!(hashtag2_prefix.as_str(), " ");
    assert_eq!(hashtag2_content.as_rule(), Rule::hashtag_content);
    assert_eq!(hashtag2_content.as_str(), "posts");

    let text3 = token_iter.next().unwrap();
    assert_eq!(text3.as_rule(), Rule::text);
    assert_eq!(text3.as_str(), "?");

    let hashtag3 = token_iter.next().unwrap();
    assert_eq!(hashtag3.as_rule(), Rule::hashtag);
    assert_eq!(hashtag3.as_str(), " #龍が如く0");

    let mut hashtag3 = hashtag3.into_inner();
    let hashtag3_prefix = hashtag3.next().unwrap();
    assert_eq!(hashtag3_prefix.as_rule(), Rule::component_prefix);
    assert_eq!(hashtag3_prefix.as_str(), " ");
    let hashtag3_content = hashtag3.next().unwrap();
    assert_eq!(hashtag3_content.as_rule(), Rule::hashtag_content);
    assert_eq!(hashtag3_content.as_str(), "龍が如く0");
}
