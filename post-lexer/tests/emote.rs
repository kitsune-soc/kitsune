use pest::Parser;
use post_lexer::{PostParser, Rule};
use pretty_assertions::assert_eq;

#[test]
fn parse_emote() {
    let text = "hello :blobfoxcoffee: how are you doing?:blobcatpeek:";
    let mut token_iter = PostParser::parse(Rule::post, text).expect("Failed to parse post");

    let text1 = token_iter.next().unwrap();
    assert_eq!(text1.as_rule(), Rule::text);
    assert_eq!(text1.as_str(), "hello ");

    let emote1 = token_iter.next().unwrap();
    assert_eq!(emote1.as_rule(), Rule::emote);
    assert_eq!(emote1.as_str(), ":blobfoxcoffee:");
    let mut emote1 = emote1.into_inner();
    let emote1_content = emote1.next().unwrap();
    assert_eq!(emote1_content.as_rule(), Rule::emote_content);
    assert_eq!(emote1_content.as_str(), "blobfoxcoffee");

    let text2 = token_iter.next().unwrap();
    assert_eq!(text2.as_rule(), Rule::text);
    assert_eq!(text2.as_str(), " how are you doing?");

    let emote2 = token_iter.next().unwrap();
    assert_eq!(emote2.as_rule(), Rule::emote);
    assert_eq!(emote2.as_str(), ":blobcatpeek:");
    let mut emote2 = emote2.into_inner();
    let emote2_content = emote2.next().unwrap();
    assert_eq!(emote2_content.as_rule(), Rule::emote_content);
    assert_eq!(emote2_content.as_str(), "blobcatpeek");
}
