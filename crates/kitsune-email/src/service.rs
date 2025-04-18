use crate::{MailSender, mails::confirm_account::ConfirmAccount};
use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{PgPool, function::now, model::user::User, schema::users, with_connection};
use kitsune_derive::kitsune_service;
use kitsune_error::Result;
use kitsune_url::UrlService;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use speedy_uuid::Uuid;

#[kitsune_service]
pub struct Mailing {
    db_pool: PgPool,
    sender: Option<MailSender<AsyncSmtpTransport<Tokio1Executor>>>,
    url_service: UrlService,
}

impl Mailing {
    #[must_use]
    pub fn has_sender(&self) -> bool {
        self.sender.is_some()
    }

    #[must_use]
    pub fn sender(&self) -> Option<MailSender<AsyncSmtpTransport<Tokio1Executor>>> {
        self.sender.clone()
    }

    pub async fn mark_as_confirmed(&self, user_id: Uuid) -> Result<()> {
        with_connection!(self.db_pool, |db_conn| {
            diesel::update(users::table.find(user_id))
                .set(users::confirmed_at.eq(now().nullable()))
                .execute(db_conn)
                .await
        })?;

        Ok(())
    }

    pub async fn mark_as_confirmed_by_token(&self, confirmation_token: &str) -> Result<()> {
        with_connection!(self.db_pool, |db_conn| {
            diesel::update(
                users::table
                    .filter(users::confirmation_token.eq(confirmation_token))
                    .filter(users::confirmed_at.is_null()),
            )
            .set(users::confirmed_at.eq(now().nullable()))
            .execute(db_conn)
            .await
        })?;

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
            let address = user.email.parse()?;

            sender.send_one(address, &mail).await?;
        }

        Ok(())
    }
}
