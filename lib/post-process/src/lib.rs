//!
//! Parser and transformer intended for usage in the Kitsune social media server
//!
//! **Important**: None of the texts are protected against XSS attacks. Keep that in mind.
//!

#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use futures_util::{pin_mut, stream, StreamExt};
use logos::{Lexer, Logos};
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use std::{borrow::Cow, error::Error, future::Future, marker::PhantomData};

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Result type with the error branch defaulting to [`BoxError`]
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

fn enforce_prefix<'a>(lexer: &Lexer<'a, LogosLexer<'a>>) -> bool {
    let start = lexer.span().start;
    if start == 0 {
        true
    } else {
        lexer.source().as_bytes()[start - 1].is_ascii_whitespace()
    }
}

fn mention_split<'a>(lexer: &Lexer<'a, LogosLexer<'a>>) -> Option<(&'a str, Option<&'a str>)> {
    if !enforce_prefix(lexer) {
        return None;
    }

    let slice = lexer.slice();
    let slice = slice.trim_start_matches('@');

    let mention_data = if let Some((username, domain)) = slice.split_once('@') {
        (username, Some(domain))
    } else {
        (slice, None)
    };

    Some(mention_data)
}

#[derive(Debug, Logos, PartialEq)]
pub enum LogosLexer<'a> {
    #[regex(r"#[\w_-]+", enforce_prefix)]
    Hashtag,

    #[regex(r"@[\w\-_]+(@[\w\-_]+\.\w+)?", mention_split)]
    Username((&'a str, Option<&'a str>)),

    #[regex(r":[\w\d-]+:")]
    Emote,

    #[regex(r"[\w]+://[^\s]+")]
    Link,
}

/// Pest-based parser
#[derive(Parser)]
#[grammar = "../grammar/post.pest"]
pub struct PostParser;

/// Post transformer
///
/// Transforms elements of a post into other elements
#[derive(Clone)]
pub struct Transformer<'a, F, T>
where
    F: Fn(Element<'a>) -> T,
    T: Future<Output = Result<Element<'a>>>,
{
    transformation: F,
    _lt: PhantomData<&'a ()>,
}

impl<'a, F, T> Transformer<'a, F, T>
where
    F: Fn(Element<'a>) -> T,
    T: Future<Output = Result<Element<'a>>>,
{
    /// Create a new transformer from a transformation function
    pub fn new(transformation: F) -> Self {
        Self {
            transformation,
            _lt: PhantomData,
        }
    }

    /// Transform a post
    ///
    /// # Errors
    ///
    /// - Transformation of an element fails
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please submit an issue
    pub async fn transform(&self, text: &'a str) -> Result<String> {
        let transformed = {
            let pairs = PostParser::parse(Rule::post, text).unwrap();
            let elements: Vec<Element<'a>> = Element::from_pairs(pairs).collect();
            stream::iter(elements).then(&self.transformation)
        };

        pin_mut!(transformed);

        let mut out = String::new();
        while let Some(elem) = transformed.next().await.transpose()? {
            elem.render(&mut out);
        }

        Ok(out)
    }
}

/// Render something into a string
pub trait Render {
    /// Render the element into its string representation
    fn render(&self, out: &mut String);
}

/// Elements of a post
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Element<'a> {
    /// Emote
    Emote(Emote<'a>),

    /// Hashtag
    Hashtag(Hashtag<'a>),

    /// Raw HTML
    Html(Html<'a>),

    /// Link
    Link(Link<'a>),

    /// Mention
    Mention(Mention<'a>),

    /// Text
    Text(Text<'a>),
}

impl<'a> Element<'a> {
    /// Generate a bunch of elements from their `Pairs` representation
    ///
    /// # Panics
    ///
    /// This should never panic. If it ever does, please submit an issue.
    pub fn from_pairs(pairs: Pairs<'a, Rule>) -> impl Iterator<Item = Element<'a>> {
        pairs.flat_map(|pair| match pair.as_rule() {
            Rule::emote => {
                let content = pair.into_inner().next().unwrap();

                vec![Self::Emote(Emote {
                    content: Cow::Borrowed(content.as_str()),
                })]
            }
            Rule::hashtag => {
                let mut hashtag = pair.into_inner();
                let prefix = hashtag.next().unwrap();
                let content = hashtag.next().unwrap();

                vec![
                    Self::Text(Text {
                        content: Cow::Borrowed(prefix.as_str()),
                    }),
                    Self::Hashtag(Hashtag {
                        content: Cow::Borrowed(content.as_str()),
                    }),
                ]
            }
            Rule::mention => {
                let mut mention = pair.into_inner();
                let prefix = mention.next().unwrap();
                let username = mention.next().unwrap();
                let domain = mention.next().map(|domain| Cow::Borrowed(domain.as_str()));

                vec![
                    Self::Text(Text {
                        content: Cow::Borrowed(prefix.as_str()),
                    }),
                    Self::Mention(Mention {
                        username: Cow::Borrowed(username.as_str()),
                        domain,
                    }),
                ]
            }
            Rule::text => vec![Self::Text(Text {
                content: Cow::Borrowed(pair.as_str()),
            })],
            Rule::link => {
                vec![Self::Link(Link {
                    content: Cow::Borrowed(pair.as_str()),
                })]
            }
            _ => unreachable!(),
        })
    }
}

impl Render for Element<'_> {
    fn render(&self, out: &mut String) {
        match self {
            Self::Emote(emote) => emote.render(out),
            Self::Hashtag(hashtag) => hashtag.render(out),
            Self::Html(html) => html.render(out),
            Self::Link(link) => link.render(out),
            Self::Mention(mention) => mention.render(out),
            Self::Text(text) => text.render(out),
        }
    }
}

/// Emote data
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Emote<'a> {
    /// Name of an emote
    pub content: Cow<'a, str>,
}

impl Render for Emote<'_> {
    fn render(&self, out: &mut String) {
        out.push(':');
        out.push_str(&self.content);
        out.push(':');
    }
}

/// Hashtag
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hashtag<'a> {
    /// Hashtag name
    pub content: Cow<'a, str>,
}

impl Render for Hashtag<'_> {
    fn render(&self, out: &mut String) {
        out.push('#');
        out.push_str(&self.content);
    }
}

/// Raw HTML
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Html<'a> {
    /// Tag name
    pub tag: Cow<'a, str>,

    /// Tag attributes
    pub attributes: Vec<(Cow<'a, str>, Cow<'a, str>)>,

    /// Tag contents
    pub content: Box<Element<'a>>,
}

impl Render for Html<'_> {
    fn render(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.tag);

        for (name, value) in &self.attributes {
            out.push(' ');
            out.push_str(name);
            out.push_str("=\"");
            out.push_str(value);
            out.push('"');
        }

        out.push('>');

        self.content.render(out);

        out.push_str("</");
        out.push_str(&self.tag);
        out.push('>');
    }
}

/// Link
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Link<'a> {
    /// Content
    pub content: Cow<'a, str>,
}

impl Render for Link<'_> {
    fn render(&self, out: &mut String) {
        out.push_str(&self.content);
    }
}

/// Mention
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mention<'a> {
    /// Username component
    pub username: Cow<'a, str>,

    /// Domain component
    pub domain: Option<Cow<'a, str>>,
}

impl Render for Mention<'_> {
    fn render(&self, out: &mut String) {
        out.push('@');
        out.push_str(&self.username);

        if let Some(ref domain) = self.domain {
            out.push('@');
            out.push_str(domain);
        }
    }
}

/// Text
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Text<'a> {
    /// Text contents
    pub content: Cow<'a, str>,
}

impl Render for Text<'_> {
    fn render(&self, out: &mut String) {
        out.push_str(&self.content);
    }
}

#[cfg(test)]
mod test {
    use crate::LogosLexer;
    use logos::Logos;

    #[test]
    fn logos_link() {
        let mut test = LogosLexer::lexer("https://github.com/kitsune-soc/kitsune    ");

        assert_eq!(test.next(), Some(Ok(LogosLexer::Link)));
        assert_eq!(test.slice(), "https://github.com/kitsune-soc/kitsune");
    }

    #[test]
    fn logos_emote() {
        let mut test = LogosLexer::lexer(":hello:");

        assert_eq!(test.next(), Some(Ok(LogosLexer::Emote)));
        assert_eq!(test.slice(), ":hello:");

        assert_eq!(test.next(), None);
    }

    #[test]
    fn logos_hashtag() {
        let mut test = LogosLexer::lexer("\n#test #龍が如く0");

        assert_eq!(test.next(), Some(Err(())));

        assert_eq!(test.next(), Some(Ok(LogosLexer::Hashtag)));
        assert_eq!(test.slice(), "#test");

        assert_eq!(test.next(), Some(Err(())));

        assert_eq!(test.next(), Some(Ok(LogosLexer::Hashtag)));
        assert_eq!(test.slice(), "#龍が如く0");

        assert_eq!(test.next(), None);
    }

    #[test]
    fn logos_mention() {
        let mut test = LogosLexer::lexer("@test");

        assert_eq!(test.next(), Some(Ok(LogosLexer::Username(("test", None)))));
        assert_eq!(test.next(), None);

        let mut test = LogosLexer::lexer("@test@example.org");
        assert_eq!(
            test.next(),
            Some(Ok(LogosLexer::Username(("test", Some("example.org")))))
        );
    }
}
