use crate::error::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{model::user::User, schema::users, PgPool};
use kitsune_email::{
    lettre::{AsyncSmtpTransport, Tokio1Executor},
    mails::confirm_account::ConfirmAccount,
    MailSender,
};
use kitsune_url::UrlService;
use scoped_futures::ScopedFutureExt;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct MailingService {
    db_pool: PgPool,
    sender: Option<MailSender<AsyncSmtpTransport<Tokio1Executor>>>,
    url_service: UrlService,
}

impl MailingService {
    #[must_use]
    pub fn has_sender(&self) -> bool {
        self.sender.is_some()
    }

    pub async fn mark_as_confirmed_by_token(&self, confirmation_token: &str) -> Result<()> {
        self.db_pool
            .with_connection(|db_conn| {
                diesel::update(
                    users::table
                        .filter(users::confirmation_token.eq(confirmation_token))
                        .filter(users::confirmed_at.is_null()),
                )
                .set(users::confirmed_at.eq(Timestamp::now_utc()))
                .execute(db_conn)
                .scoped()
            })
            .await?;

        Ok(())
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
