use pulldown_cmark::{html, Options, Parser};

#[inline]
#[must_use]
pub fn markdown(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut buf = String::new();
    html::push_html(&mut buf, parser);
    buf
}
