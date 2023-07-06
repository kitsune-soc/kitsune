#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use crate::{
    error::{Error, Result},
    traits::{MailingBackend, RenderableEmail},
};
use std::iter;
use typed_builder::TypedBuilder;

pub mod backend;
pub mod error;
pub mod mails;
pub mod traits;

#[derive(TypedBuilder)]
pub struct Sender<B> {
    backend: B,
}

impl<B> Sender<B>
where
    B: MailingBackend,
{
    pub async fn send<'a, I, M>(&self, addresses: I, email: &M) -> Result<()>
    where
        I: Iterator<Item = &'a str> + Send,
        M: RenderableEmail,
    {
        let rendered_email = email.render_email()?;
        self.backend
            .send_email(addresses, rendered_email)
            .await
            .map_err(|err| Error::MailingBackend(err.into()))
    }

    pub async fn send_one<M>(&self, address: &str, email: &M) -> Result<()>
    where
        M: RenderableEmail,
    {
        self.send(iter::once(address), email).await
    }
}
