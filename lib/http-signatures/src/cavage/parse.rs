use super::SignatureHeader;
use logos::{Lexer, Logos, Span};

#[derive(Debug, Logos)]
enum TokenTy {
    #[regex(r"\w+")]
    Key,

    #[token("=")]
    Equals,

    #[regex(r#""[^"]*""#)]
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
    pub fn parse(input: &str) -> impl Iterator<Item = Result<Token, ()>> + '_ {
        Lexer::<'_, TokenTy>::new(input)
            .spanned()
            .map(|(ty, span)| ty.map(|ty| Token { ty, span }))
    }
}

macro_rules! ensure {
    ($self:expr, $value:expr, $pattern:pat) => {{
        let Ok(value) = $value else {
            $self.is_broken = true;
            return Some(Err(()));
        };

        if !matches!(value.ty, $pattern) {
            $self.is_broken = true;
            return Some(Err(()));
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
    I: Iterator<Item = Result<Token, ()>>,
{
    type Item = Result<(&'a str, &'a str), ()>;

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

        // TODO: We can technically replace this indexing with an unchecked index since we have the same input the lexer had.
        //       Could skip some unnecessary branches and some unnecessary checks.
        let key = &self.input[key.span];
        let value = self.input[value.span].trim_matches('"');

        Some(Ok((key, value)))
    }
}

/// Parse a cavage `Signature` header into key/value pairs with proper error handling
#[inline]
pub fn parse(input: &str) -> Result<SignatureHeader<'_, impl Iterator<Item = &str>>, ()> {
    let kv_iter = ParseIter {
        inner: Token::parse(input),
        input,
        is_broken: false,
    };

    // TODO: Maybe replace this with `derive_builder`? Not sure. That would definitely pull in `syn` v1 as a dependency.
    let mut key_id = None;
    let mut signature = None;
    let mut headers = None;
    let mut created = None;
    let mut expires = None;

    for kv in kv_iter {
        let (key, value) = kv?;

        match key {
            "algorithm" => {
                // We just discard this value and ignore it
                // It doesn't really matter anymore. We just figure the algorithm type out via the key algorithm identifier
            }
            "keyId" => key_id = Some(value),
            "signature" => signature = Some(value),
            "headers" => headers = Some(value.split_whitespace()),
            "created" => created = Some(atoi_radix10::parse_from_str(value).map_err(|_| ())?),
            "expires" => expires = Some(atoi_radix10::parse_from_str(value).map_err(|_| ())?),
            _ => return Err(()),
        }
    }

    Ok(SignatureHeader {
        key_id: key_id.ok_or(())?,
        signature: signature.ok_or(())?,
        headers: headers.ok_or(())?,
        created,
        expires,
    })
}

#[cfg(test)]
mod test {
    use super::parse;

    const HEADER: &str = r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#;

    #[test]
    fn parse_header() {
        let header = parse(HEADER).unwrap();

        assert_eq!(header.created, None);
        assert_eq!(header.expires, None);
        assert_eq!(header.key_id, "Test");
        assert_eq!(header.signature, "qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=");
        assert_eq!(
            header.headers.collect::<Vec<_>>(),
            ["(request-target)", "host", "date"]
        );
    }
}
