use crate::traits::RenderableEmail;
use kitsune_error::{Error, Result};
use lettre::{
    message::{Mailbox, MultiPart},
    AsyncTransport, Message,
};
use std::{iter, sync::Arc};
use typed_builder::TypedBuilder;

pub use self::service::Mailing as MailingService;
pub use lettre;

pub mod mails;
pub mod service;
pub mod traits;

#[derive(Clone, TypedBuilder)]
pub struct MailSender<B> {
    backend: B,
    #[builder(setter(into))]
    from_mailbox: Arc<Mailbox>,
}

impl<B> MailSender<B>
where
    B: AsyncTransport + Sync,
    Error: From<<B as AsyncTransport>::Error>,
{
    pub async fn send<'a, I, M>(&self, mailboxes: I, email: &M) -> Result<()>
    where
        I: Iterator<Item = Mailbox> + Send,
        M: RenderableEmail,
    {
        let rendered_email = email.render_email()?;
        for mailbox in mailboxes {
            let message = Message::builder()
                .from(self.from_mailbox.as_ref().clone())
                .to(mailbox)
                .subject(rendered_email.subject.as_str())
                // I love email. I love how it is stuck in the 70s. So cute.
                .multipart(MultiPart::alternative_plain_html(
                    rendered_email.plain_text.clone(),
                    rendered_email.body.clone(),
                ))?;

            self.backend.send(message).await?;
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
