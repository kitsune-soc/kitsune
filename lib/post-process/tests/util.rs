use logos::{Lexer, Span};
use post_process::LogosLexer;

pub fn parse_to_test_output(src: &str) -> Vec<(LogosLexer, Span, &str)> {
    Lexer::new(src)
        .spanned()
        .flat_map(|(token, span)| token.map(|token| (token, span)))
        .map(|(token, span)| (token, span.clone(), &src[span]))
        .collect()
}
