use crate::error::{BoxError, Result};
use async_trait::async_trait;

#[derive(Debug)]
pub struct RenderedEmail {
    pub subject: String,
    pub body: String,
}

pub trait RenderableEmail {
    fn render_email(&self) -> Result<RenderedEmail>;
}

#[async_trait]
pub trait MailingBackend {
    type Error: Into<BoxError>;

    async fn send_email<'a, I>(
        &self,
        addresses: I,
        email: RenderedEmail,
    ) -> Result<(), Self::Error>
    where
        I: Iterator<Item = &'a str> + Send;
}
