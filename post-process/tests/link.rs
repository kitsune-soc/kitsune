use pest::Parser;
use post_process::{PostParser, Rule};
use pretty_assertions::assert_eq;

#[test]
fn link_only() {
    let link = "https://upload.wikimedia.org/wikipedia/en/3/3b/GoroMajimafive.jpg";
    let mut pairs = PostParser::parse(Rule::link, link).unwrap();

    let mut link = pairs.next().unwrap().into_inner();
    let schema = link.next().unwrap();
    let content = link.next().unwrap();

    assert_eq!(schema.as_rule(), Rule::link_schema);
    assert_eq!(schema.as_str(), "https");

    assert_eq!(content.as_rule(), Rule::link_content);
    assert_eq!(
        content.as_str(),
        "upload.wikimedia.org/wikipedia/en/3/3b/GoroMajimafive.jpg"
    );
}

#[test]
fn mixed_content() {
    let text = "hey, @真島 looking good.. #龍が如く7 https://upload.wikimedia.org/wikipedia/en/3/3b/GoroMajimafive.jpg";
    let mut pairs = PostParser::parse(Rule::post, text).unwrap();

    let text1 = pairs.next().unwrap();
    assert_eq!(text1.as_rule(), Rule::text);
    assert_eq!(text1.as_str(), "hey,");

    let mention = pairs.next().unwrap();
    assert_eq!(mention.as_rule(), Rule::mention);
    assert_eq!(mention.as_str(), " @真島");

    let text2 = pairs.next().unwrap();
    assert_eq!(text2.as_rule(), Rule::text);
    assert_eq!(text2.as_str(), " looking good..");

    let hashtag = pairs.next().unwrap();
    assert_eq!(hashtag.as_rule(), Rule::hashtag);
    assert_eq!(hashtag.as_str(), " #龍が如く7");

    let text3 = pairs.next().unwrap();
    assert_eq!(text3.as_rule(), Rule::text);
    assert_eq!(text3.as_str(), " ");

    let link = pairs.next().unwrap();
    assert_eq!(link.as_rule(), Rule::link);
    assert_eq!(
        link.as_str(),
        "https://upload.wikimedia.org/wikipedia/en/3/3b/GoroMajimafive.jpg"
    );

    let mut link = link.into_inner();
    let schema = link.next().unwrap();
    assert_eq!(schema.as_rule(), Rule::link_schema);
    assert_eq!(schema.as_str(), "https");

    let content = link.next().unwrap();
    assert_eq!(content.as_rule(), Rule::link_content);
    assert_eq!(
        content.as_str(),
        "upload.wikimedia.org/wikipedia/en/3/3b/GoroMajimafive.jpg"
    );
}
