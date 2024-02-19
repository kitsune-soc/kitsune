use super::{SignatureHeader, SignatureHeaderBuilder, SignatureHeaderBuilderError};
use logos::{Lexer, Logos, Span};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum ParseError {
    #[error("Invalid sequence")]
    InvalidSequence {
        #[label("This stuff")]
        span: SourceSpan,
    },

    #[error("Missing field: {0}")]
    MissingField(&'static str),

    #[error("Radix 10 value parsing failed")]
    Radix10Parse,

    #[error("Unexpected token")]
    UnexpectedToken {
        got: TokenTy,
        expected: TokenTy,
        #[label("Expected: {expected:?}, got: {got:?}")]
        span: SourceSpan,
    },
}

#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \n]+")]
pub enum TokenTy {
    #[regex(r"[a-zA-Z]+")]
    Key,

    #[token("=")]
    Equals,

    #[regex(r#""[^"]*"|[0-9]+"#)]
    Value,

    #[token(",")]
    Comma,
}

#[derive(Debug)]
struct Token {
    pub ty: TokenTy,
    pub span: Span,
}

impl Token {
    pub fn parse(input: &str) -> impl Iterator<Item = Result<Token, ParseError>> + '_ {
        Lexer::<'_, TokenTy>::new(input)
            .spanned()
            .map(|(ty, span)| {
                ty.map({
                    let span = span.clone();
                    |ty| Token { ty, span }
                })
                .map_err(|()| ParseError::InvalidSequence { span: span.into() })
            })
    }
}

macro_rules! ensure {
    ($self:expr, $value:expr, $pattern:expr) => {{
        let value = match $value {
            Ok(val) => val,
            Err(err) => {
                $self.is_broken = true;
                return Some(Err(err));
            }
        };

        if value.ty != $pattern {
            $self.is_broken = true;
            return Some(Err(ParseError::UnexpectedToken {
                got: value.ty,
                expected: $pattern,
                span: value.span.into(),
            }));
        }

        value
    }};
}

struct ParseIter<'a, I> {
    /// Stream of tokens wrapped into a result
    inner: I,

    /// Reference to the original input that was fed to the lexer
    input: &'a str,

    /// Marker whether we encountered any error or illegal token
    ///
    /// If we did, the iterator will stop yielding any results
    is_broken: bool,
}

impl<'a, I> Iterator for ParseIter<'a, I>
where
    I: Iterator<Item = Result<Token, ParseError>>,
{
    type Item = Result<(&'a str, &'a str), ParseError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_broken {
            return None;
        }

        let key = ensure!(self, self.inner.next()?, TokenTy::Key);
        ensure!(self, self.inner.next()?, TokenTy::Equals);
        let value = ensure!(self, self.inner.next()?, TokenTy::Value);

        if let Some(next) = self.inner.next() {
            ensure!(self, next, TokenTy::Comma);
        }

        let key = &self.input[key.span];
        let value = self.input[value.span].trim_matches('"');

        Some(Ok((key, value)))
    }
}

/// Parse a cavage `Signature` header into key/value pairs with proper error handling
#[inline]
pub fn parse(
    input: &str,
) -> Result<SignatureHeader<'_, impl Iterator<Item = &str> + Clone, &str>, ParseError> {
    let mut kv_iter = ParseIter {
        inner: Token::parse(input),
        input,
        is_broken: false,
    };

    let mut builder = SignatureHeaderBuilder::default();
    while let Some((key, value)) = kv_iter.next().transpose()? {
        match key {
            "keyId" => {
                builder.key_id(value);
            }
            "signature" => {
                builder.signature(value);
            }
            "headers" => {
                builder.headers(value.split_whitespace());
            }
            "created" => {
                builder.created(
                    atoi_radix10::parse_from_str(value).map_err(|_| ParseError::Radix10Parse)?,
                );
            }
            "expires" => {
                builder.expires(
                    atoi_radix10::parse_from_str(value).map_err(|_| ParseError::Radix10Parse)?,
                );
            }
            _ => {
                // Simply discard unknown values
                //
                // Also covers the "algorithm" field since we just figure out the algorithm from the key and its algorithm identifier
            }
        }
    }

    builder.build().map_err(|err| {
        let SignatureHeaderBuilderError::UninitializedField(field_name) = err else {
            unreachable!();
        };

        ParseError::MissingField(field_name)
    })
}

#[cfg(test)]
mod test {
    use super::parse;

    const HEADER_1: &str = r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#;
    const HEADER_2: &str = r#"keyId="Test",algorithm="rsa-sha256",created=1402170695, expires=1402170699,headers="(request-target) (created) (expires) host date content-type digest content-length",signature="vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE=""#;

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn parse_header() {
        let header_1 = parse(HEADER_1).unwrap();

        assert_eq!(header_1.created, None);
        assert_eq!(header_1.expires, None);
        assert_eq!(header_1.key_id, "Test");
        assert_eq!(header_1.signature, "qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=");
        assert_eq!(
            header_1.headers.collect::<Vec<_>>(),
            ["(request-target)", "host", "date"]
        );

        let header_2 = parse(HEADER_2).unwrap();

        assert_eq!(header_2.created, Some(1402170695));
        assert_eq!(header_2.expires, Some(1402170699));
        assert_eq!(header_2.key_id, "Test");
        assert_eq!(header_2.signature, "vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE=");
        assert_eq!(
            header_2.headers.collect::<Vec<_>>(),
            [
                "(request-target)",
                "(created)",
                "(expires)",
                "host",
                "date",
                "content-type",
                "digest",
                "content-length"
            ]
        );
    }
}
