use pulldown_cmark::{Options, Parser, html};

#[inline]
#[must_use]
pub fn markdown(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut buf = String::new();
    html::push_html(&mut buf, parser);
    buf
}
