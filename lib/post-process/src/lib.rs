//!
//! Parser and transformer intended for usage in the Kitsune social media server
//!
//! **Important**: None of the texts are protected against XSS attacks. Keep that in mind.
//!

#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use logos::{Lexer, Logos, Span};
use std::{borrow::Cow, error::Error, fmt, future::Future};

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Result type with the error branch defaulting to [`BoxError`]
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

fn enforce_postfix<'a>(lexer: &Lexer<'a, PostElement<'a>>) -> bool {
    let end = lexer.span().end;
    if end == lexer.source().len() {
        true
    } else {
        !lexer.source().as_bytes()[end].is_ascii_alphanumeric()
    }
}

fn enforce_prefix<'a>(lexer: &Lexer<'a, PostElement<'a>>) -> bool {
    let start = lexer.span().start;
    if start == 0 {
        true
    } else {
        !lexer.source().as_bytes()[start - 1].is_ascii_alphanumeric()
    }
}

fn mention_split<'a>(lexer: &Lexer<'a, PostElement<'a>>) -> Option<(&'a str, Option<&'a str>)> {
    if !enforce_prefix(lexer) || !enforce_postfix(lexer) {
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
pub enum PostElement<'a> {
    #[regex(
        r":[\w\d-]+:",
        |lexer| lexer.slice().trim_matches(':'),
    )]
    Emote(&'a str),

    #[regex(
        r"#[\w_-]+",
        |lexer| enforce_prefix(lexer).then(|| lexer.slice().trim_start_matches('#')),
    )]
    Hashtag(&'a str),

    #[regex(r"@[\w\-_]+(@[\w\-_]+\.[\.\w]+)?", mention_split)]
    Mention((&'a str, Option<&'a str>)),

    #[regex(r"[\w]+://[^\s<]+")]
    Link(&'a str),
}

/// Transform a post
///
/// # Errors
///
/// - Transformation of an element fails
pub async fn transform<'a, F, Fut>(text: &'a str, transformer: F) -> Result<String>
where
    F: Fn(Element<'a>) -> Fut,
    Fut: Future<Output = Result<Element<'a>>>,
{
    let element_iter = {
        let pairs = Lexer::new(text)
            .spanned()
            .flat_map(|(token, span)| token.map(|token| (token, span)));

        Element::from_pairs(pairs)
            .collect::<Vec<(Element<'a>, Span)>>()
            .into_iter()
            .rev()
    };

    let mut out = text.to_string();
    let mut buffer = String::new();

    for (element, span) in element_iter {
        let element = transformer(element).await?;

        element.render(&mut buffer);
        out.replace_range(span, &buffer);
        buffer.clear();
    }

    Ok(out)
}

/// Render something into a string
pub trait Render {
    /// Render the element into its string representation
    fn render(&self, out: &mut impl fmt::Write);
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
    pub fn from_pairs(
        pairs: impl Iterator<Item = (PostElement<'a>, Span)>,
    ) -> impl Iterator<Item = (Element<'a>, Span)> {
        pairs.map(|(item, span)| {
            let element = match item {
                PostElement::Emote(name) => Self::Emote(Emote {
                    content: Cow::Borrowed(name),
                }),
                PostElement::Hashtag(content) => Self::Hashtag(Hashtag {
                    content: Cow::Borrowed(content),
                }),
                PostElement::Mention((username, domain)) => Self::Mention(Mention {
                    username: Cow::Borrowed(username),
                    domain: domain.map(Cow::Borrowed),
                }),
                PostElement::Link(content) => Self::Link(Link {
                    content: Cow::Borrowed(content),
                }),
            };

            (element, span)
        })
    }
}

impl Render for Element<'_> {
    fn render(&self, out: &mut impl fmt::Write) {
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
    fn render(&self, out: &mut impl fmt::Write) {
        let _ = write!(out, ":{}:", self.content);
    }
}

/// Hashtag
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hashtag<'a> {
    /// Hashtag name
    pub content: Cow<'a, str>,
}

impl Render for Hashtag<'_> {
    fn render(&self, out: &mut impl fmt::Write) {
        let _ = write!(out, "#{}", self.content);
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
    fn render(&self, out: &mut impl fmt::Write) {
        let _ = write!(out, "<{}", self.tag);
        for (name, value) in &self.attributes {
            let _ = write!(out, " {name}=\"{value}\"");
        }
        let _ = out.write_char('>');

        self.content.render(out);

        let _ = write!(out, "</{}>", self.tag);
    }
}

/// Link
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Link<'a> {
    /// Content
    pub content: Cow<'a, str>,
}

impl Render for Link<'_> {
    fn render(&self, out: &mut impl fmt::Write) {
        let _ = out.write_str(&self.content);
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
    fn render(&self, out: &mut impl fmt::Write) {
        let _ = write!(out, "@{}", self.username);

        if let Some(ref domain) = self.domain {
            let _ = write!(out, "@{domain}");
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
    fn render(&self, out: &mut impl fmt::Write) {
        let _ = out.write_str(&self.content);
    }
}
