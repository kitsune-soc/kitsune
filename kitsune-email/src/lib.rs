#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use crate::{
    error::{BoxError, Error, Result},
    traits::RenderableEmail,
};
use lettre::{
    message::{Mailbox, MultiPart},
    AsyncTransport, Message,
};
use std::iter;
use typed_builder::TypedBuilder;

pub use lettre;

pub mod error;
pub mod mails;
pub mod traits;

#[derive(Clone, TypedBuilder)]
pub struct MailSender<B> {
    backend: B,
}

impl<B> MailSender<B>
where
    B: AsyncTransport + Sync,
    <B as AsyncTransport>::Error: Into<BoxError>,
{
    pub async fn send<'a, I, M>(&self, mailboxes: I, email: &M) -> Result<()>
    where
        I: Iterator<Item = Mailbox> + Send,
        M: RenderableEmail,
    {
        let rendered_email = email.render_email()?;
        for mailbox in mailboxes {
            let message = Message::builder()
                .to(mailbox)
                .subject(rendered_email.subject.as_str())
                // I love email. I love how it is stuck in the 70s. So cute.
                .multipart(MultiPart::alternative_plain_html(
                    rendered_email.plain_text.clone(),
                    rendered_email.body.clone(),
                ))?;

            self.backend
                .send(message)
                .await
                .map_err(|err| Error::Transport(err.into()))?;
        }

        Ok(())
    }

    pub async fn send_one<M>(&self, address: Mailbox, email: &M) -> Result<()>
    where
        M: RenderableEmail,
    {
        self.send(iter::once(address), email).await
    }
}
