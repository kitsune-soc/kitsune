use crate::{error::Result, mails::confirm_account::ConfirmAccount, MailSender};
use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::{function::now, model::user::User, schema::users, PgPool};
use kitsune_url::UrlService;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
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
        self.db_pool
            .with_connection(|db_conn| {
                diesel::update(users::table.find(user_id))
                    .set(users::confirmed_at.eq(now().nullable()))
                    .execute(db_conn)
                    .scoped()
            })
            .await?;

        Ok(())
    }

    pub async fn mark_as_confirmed_by_token(&self, confirmation_token: &str) -> Result<()> {
        self.db_pool
            .with_connection(|db_conn| {
                diesel::update(
                    users::table
                        .filter(users::confirmation_token.eq(confirmation_token))
                        .filter(users::confirmed_at.is_null()),
                )
                .set(users::confirmed_at.eq(now().nullable()))
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
            let address = user.email.parse()?;

            sender.send_one(address, &mail).await?;
        }

        Ok(())
    }
}
