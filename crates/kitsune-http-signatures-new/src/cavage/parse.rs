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
        Some(Ok((&self.input[key.span], &self.input[value.span])))
    }
}

/// Parse a cavage `Signature` header into key/value pairs with proper error handling
#[inline]
pub fn parse(input: &str) -> impl Iterator<Item = Result<(&str, &str), ()>> {
    ParseIter {
        inner: Token::parse(input),
        input,
        is_broken: false,
    }
}
