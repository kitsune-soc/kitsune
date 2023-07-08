use crate::error::Result;

#[derive(Debug)]
pub struct RenderedEmail {
    pub subject: String,
    pub body: String,

    /// Ugly plain text fallback for the clients that refuse to render our absolutely stunning HTML
    pub plain_text: String,
}

pub trait RenderableEmail {
    fn render_email(&self) -> Result<RenderedEmail>;
}
