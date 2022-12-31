use pest_derive::Parser;

/// Pest-based parser
#[derive(Parser)]
#[grammar = "../grammar/post.pest"]
pub struct PostParser;

#[cfg(test)]
mod test {
    use crate::{PostParser, Rule};
    use pest::Parser;

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

        let text2 = token_iter.next().unwrap();
        assert_eq!(text2.as_rule(), Rule::text);
        assert_eq!(text2.as_str(), " how are you doing?");

        let emote2 = token_iter.next().unwrap();
        assert_eq!(emote2.as_rule(), Rule::emote);
        assert_eq!(emote2.as_str(), ":blobcatpeek:");
    }

    #[test]
    fn parse_hashtag() {
        let text = "why am i building a #lexer for #posts?";
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
        let hashtag1_postfix = hashtag1.next().unwrap();

        assert_eq!(hashtag1_prefix.as_rule(), Rule::component_prefix);
        assert_eq!(hashtag1_prefix.as_str(), " ");
        assert_eq!(hashtag1_content.as_rule(), Rule::hashtag_content);
        assert_eq!(hashtag1_content.as_str(), "lexer");
        assert_eq!(hashtag1_postfix.as_rule(), Rule::component_postfix);

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
    }

    #[test]
    fn parse_mention() {
        let text = "hello @桐生@friday.night @真島 ! ";
        let mut token_iter = PostParser::parse(Rule::post, text).expect("Failed to parse post");

        let text1 = token_iter.next().unwrap();
        assert_eq!(text1.as_rule(), Rule::text);
        assert_eq!(text1.as_str(), "hello");

        let mention1 = token_iter.next().unwrap();
        assert_eq!(mention1.as_rule(), Rule::mention);
        assert_eq!(mention1.as_str(), " @桐生@friday.night");

        let mut mention1 = mention1.into_inner();
        let mention_prefix = mention1.next().unwrap();
        assert_eq!(mention_prefix.as_rule(), Rule::component_prefix);
        assert_eq!(mention_prefix.as_str(), " ");
        let mention_username1 = mention1.next().unwrap();
        assert_eq!(mention_username1.as_rule(), Rule::mention_username);
        assert_eq!(mention_username1.as_str(), "桐生");
        let mention_domain1 = mention1.next().unwrap();
        assert_eq!(mention_domain1.as_rule(), Rule::mention_domain);
        assert_eq!(mention_domain1.as_str(), "friday.night");
        let mention_postfix = mention1.next().unwrap();
        assert_eq!(mention_postfix.as_rule(), Rule::component_postfix);
        assert_eq!(mention_postfix.as_str(), "");
    }
}
