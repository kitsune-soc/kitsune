//!
//! Parser and transformer intended for usage in the Kitsune social media server
//!
//! **Important**: None of the texts are protected against XSS attacks. Keep that in mind.
//!

#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use futures_util::{pin_mut, stream, StreamExt};
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use std::{borrow::Cow, error::Error, future::Future, marker::PhantomData};

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Result type with the error branch defaulting to [`BoxError`]
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

/// Pest-based parser
#[derive(Parser)]
#[grammar = "../grammar/post.pest"]
pub struct PostParser;

/// Post transformer
///
/// Transforms elements of a post into other elements
#[derive(Clone)]
pub struct Transformer<'a, F, Fut>
where
    F: Fn(Element<'a>) -> Fut,
    Fut: Future<Output = Result<Element<'a>>> + Send,
{
    transformation: F,
    _fut: PhantomData<&'a Fut>,
}

impl<'a, F, Fut> Transformer<'a, F, Fut>
where
    F: Fn(Element<'a>) -> Fut,
    Fut: Future<Output = Result<Element<'a>>> + Send,
{
    /// Create a new transformer from a transformation function
    pub fn new(transformation: F) -> Self {
        Self {
            transformation,
            _fut: PhantomData,
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
        let pairs = PostParser::parse(Rule::post, text).unwrap();
        let elements = Element::from_pairs(pairs);
        let transformed = stream::iter(elements).then(&self.transformation);

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
