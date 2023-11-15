use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::user::User, schema::users};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct SendConfirmationMail {
    pub user_id: Uuid,
}

impl<D> Runnable for SendConfirmationMail {
    type Context = JobRunnerContext<D>;
    type Error = eyre::Report;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let (mailing_service, user_service) = (&ctx.state.service.mailing, &ctx.state.service.user);

        // If we don't have a mailer, just don't bother and mark the user as confirmed
        if !mailing_service.has_sender() {
            user_service.mark_as_confirmed(self.user_id).await?;
        }

        let user = ctx
            .state
            .db_pool
            .with_connection(|db_conn| {
                users::table
                    .find(self.user_id)
                    .select(User::as_select())
                    .get_result(db_conn)
                    .scoped()
            })
            .await?;

        mailing_service.send_confirmation_email(&user).await?;

        Ok(())
    }
}
