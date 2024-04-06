use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::user::User, schema::users, with_connection};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct SendConfirmationMail {
    pub user_id: Uuid,
}

impl Runnable for SendConfirmationMail {
    type Context = JobRunnerContext;
    type Error = kitsune_error::Error;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let mailing_service = &ctx.service.mailing;

        // If we don't have a mailer, just don't bother and mark the user as confirmed
        if !mailing_service.has_sender() {
            mailing_service.mark_as_confirmed(self.user_id).await?;
        }

        let user = with_connection!(ctx.db_pool, |db_conn| {
            users::table
                .find(self.user_id)
                .select(User::as_select())
                .get_result(db_conn)
                .await
        })?;

        mailing_service.send_confirmation_email(&user).await?;

        Ok(())
    }
}
