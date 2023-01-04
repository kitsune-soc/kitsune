use pest::Parser;
use post_process::{PostParser, Rule};
use pretty_assertions::assert_eq;

#[test]
fn invalid_mention() {
    let text = "@test@hello.world@tes";
    assert!(PostParser::parse(Rule::mention, text).is_err());
}

#[test]
fn another_invalid_mention() {
    let text = "@ test@hello.world";
    assert!(PostParser::parse(Rule::mention, text).is_err());
}

#[test]
fn weird_invalid_mention() {
    let text = "@test@hello.world@tes hello";
    let mut token_iter = PostParser::parse(Rule::post, text).unwrap();

    let text1 = token_iter.next().unwrap();
    assert_eq!(text1.as_rule(), Rule::text);
    assert_eq!(text1.as_str(), "@test@hello.world@tes hello");

    assert!(token_iter.next().is_none());
}

#[test]
fn parse_mention() {
    let text = "hello @桐生@friday.night @真島! ";
    let mut token_iter = PostParser::parse(Rule::post, text).expect("Failed to parse post");

    let text1 = token_iter.next().unwrap();
    assert_eq!(text1.as_rule(), Rule::text);
    assert_eq!(text1.as_str(), "hello");

    let mention1 = token_iter.next().unwrap();
    assert_eq!(mention1.as_rule(), Rule::mention);
    assert_eq!(mention1.as_str(), " @桐生@friday.night");

    let mut mention1 = mention1.into_inner();
    let mention1_prefix = mention1.next().unwrap();
    assert_eq!(mention1_prefix.as_rule(), Rule::component_prefix);
    assert_eq!(mention1_prefix.as_str(), " ");
    let mention1_username = mention1.next().unwrap();
    assert_eq!(mention1_username.as_rule(), Rule::mention_username);
    assert_eq!(mention1_username.as_str(), "桐生");
    let mention1_domain = mention1.next().unwrap();
    assert_eq!(mention1_domain.as_rule(), Rule::mention_domain);
    assert_eq!(mention1_domain.as_str(), "friday.night");

    let mention2 = token_iter.next().unwrap();
    assert_eq!(mention2.as_rule(), Rule::mention);
    assert_eq!(mention2.as_str(), " @真島");
    let mut mention2 = mention2.into_inner();
    let mention2_prefix = mention2.next().unwrap();
    assert_eq!(mention2_prefix.as_rule(), Rule::component_prefix);
    assert_eq!(mention2_prefix.as_str(), " ");
    let mention2_username = mention2.next().unwrap();
    assert_eq!(mention2_username.as_rule(), Rule::mention_username);
    assert_eq!(mention2_username.as_str(), "真島");

    let text2 = token_iter.next().unwrap();
    assert_eq!(text2.as_rule(), Rule::text);
    assert_eq!(text2.as_str(), "! ");
}
