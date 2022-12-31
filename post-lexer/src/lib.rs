#![forbid(rust_2018_idioms)]

use async_stream::stream;
use futures_util::{future::BoxFuture, pin_mut, Stream, StreamExt};
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use std::{borrow::Cow, error::Error};

pub type BoxError = Box<dyn Error + Send + Sync>;
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

/// Pest-based parser
#[derive(Parser)]
#[grammar = "../grammar/post.pest"]
pub struct PostParser;

pub struct PostTransformer {
    transformer: Transformer,
}

impl PostTransformer {
    pub fn new(transformer: Transformer) -> Self {
        Self { transformer }
    }

    pub async fn transform(&self, text: &str) -> Result<String> {
        let pairs = PostParser::parse(Rule::post, text).unwrap();
        let elements = Element::from_pairs(pairs);
        let transformed = self.transformer.transform(elements);

        pin_mut!(transformed);

        let mut out = String::new();
        while let Some(elem) = transformed.next().await.transpose()? {
            elem.render(&mut out);
        }

        Ok(out)
    }
}

#[derive(Clone)]
pub struct Transformer {
    transformation: fn(Element<'_>) -> BoxFuture<'_, Result<Element<'_>>>,
}

impl Transformer {
    pub fn new(transformation: fn(Element<'_>) -> BoxFuture<'_, Result<Element<'_>>>) -> Self {
        Self { transformation }
    }

    pub fn transform<'a, E>(&'a self, elems: E) -> impl Stream<Item = Result<Element<'a>>>
    where
        E: Iterator<Item = Element<'a>>,
    {
        stream! {
            for elem in elems {
                yield (self.transformation)(elem).await;
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Element<'a> {
    Emote(Emote<'a>),
    Hashtag(Hashtag<'a>),
    Html(Html<'a>),
    Mention(Mention<'a>),
    Text(Text<'a>),
}

impl<'a> Element<'a> {
    /// Generate a bunch of elements from their `Pairs` representation
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

    pub fn render(self, out: &mut String) {
        match self {
            Self::Emote(emote) => emote.render(out),
            Self::Hashtag(hashtag) => hashtag.render(out),
            Self::Html(html) => html.render(out),
            Self::Mention(mention) => mention.render(out),
            Self::Text(text) => text.render(out),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Emote<'a> {
    pub content: Cow<'a, str>,
}

impl Emote<'_> {
    pub fn render(self, out: &mut String) {
        out.push(':');
        out.push_str(&self.content);
        out.push(':');
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hashtag<'a> {
    pub content: Cow<'a, str>,
}

impl Hashtag<'_> {
    pub fn render(self, out: &mut String) {
        out.push('#');
        out.push_str(&self.content);
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Html<'a> {
    pub tag: Cow<'a, str>,
    pub attributes: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    pub content: Box<Element<'a>>,
}

impl Html<'_> {
    pub fn render(self, out: &mut String) {
        out.push('<');
        out.push_str(&self.tag);

        for (name, value) in self.attributes {
            out.push(' ');
            out.push_str(&name);
            out.push_str("=\"");
            out.push_str(&value);
            out.push('"');
        }

        out.push('>');

        self.content.render(out);

        out.push_str("</");
        out.push_str(&self.tag);
        out.push('>');
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Mention<'a> {
    pub username: Cow<'a, str>,
    pub domain: Option<Cow<'a, str>>,
}

impl Mention<'_> {
    pub fn render(self, out: &mut String) {
        out.push('@');
        out.push_str(&self.username);

        if let Some(domain) = self.domain {
            out.push('@');
            out.push_str(&domain);
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Text<'a> {
    pub content: Cow<'a, str>,
}

impl Text<'_> {
    pub fn render(self, out: &mut String) {
        out.push_str(&self.content);
    }
}
