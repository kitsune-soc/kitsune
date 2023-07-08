use super::url::UrlService;
use crate::error::Result;
use kitsune_db::model::user::User;
use kitsune_email::{
    lettre::{AsyncSmtpTransport, Tokio1Executor},
    mails::confirm_account::ConfirmAccount,
    MailSender,
};
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct MailingService {
    sender: Option<MailSender<AsyncSmtpTransport<Tokio1Executor>>>,
    url_service: UrlService,
}

impl MailingService {
    #[must_use]
    pub fn has_sender(&self) -> bool {
        self.sender.is_some()
    }

    pub async fn send_confirmation_email(&self, user: &User) -> Result<()> {
        let verify_link = self
            .url_service
            .confirm_account_url(user.confirmation_token.as_str());

        let mail = ConfirmAccount::builder()
            .domain(self.url_service.domain())
            .username(user.username.as_str())
            .verify_link(verify_link.as_str())
            .build();

        if let Some(ref sender) = self.sender {
            let address = user
                .email
                .parse()
                .map_err(kitsune_email::error::Error::from)?;

            sender.send_one(address, &mail).await?;
        }

        Ok(())
    }
}
