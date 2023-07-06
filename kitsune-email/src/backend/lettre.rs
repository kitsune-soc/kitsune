use crate::traits::{MailingBackend, RenderedEmail};
use async_trait::async_trait;
use lettre::{
    address::AddressError,
    message::{Mailbox, MultiPart, SinglePart},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::str::FromStr;
use thiserror::Error;
use typed_builder::TypedBuilder;

pub use lettre;

#[derive(Debug, Error)]
pub enum LettreBackendError {
    #[error(transparent)]
    Address(#[from] AddressError),

    #[error(transparent)]
    Lettre(#[from] lettre::error::Error),

    #[error(transparent)]
    Smtp(#[from] lettre::transport::smtp::Error),
}

#[derive(TypedBuilder)]
pub struct LettreBackend {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

#[async_trait]
impl MailingBackend for LettreBackend {
    type Error = LettreBackendError;

    async fn send_email<'a, I>(&self, addresses: I, email: RenderedEmail) -> Result<(), Self::Error>
    where
        I: Iterator<Item = &'a str> + Send,
    {
        let mut mailboxes = addresses.map(Mailbox::from_str);

        while let Some(mailbox) = mailboxes.next().transpose()? {
            let message = Message::builder()
                .to(mailbox)
                .subject(email.subject.as_str())
                .multipart(MultiPart::builder().singlepart(SinglePart::html(email.body.clone())))?;

            self.transport.send(message).await?;
        }

        Ok(())
    }
}
