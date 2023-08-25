use crate::{job::JobRunnerContext, mapping::IntoActivity, try_join};
use async_trait::async_trait;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, follower::Follow, user::User},
    schema::{accounts, accounts_follows, users},
};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverFollow {
    pub follow_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverFollow {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(follow_id = %self.follow_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let follow = ctx
            .state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    accounts_follows::table
                        .find(self.follow_id)
                        .get_result::<Follow>(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        let Some(follow) = follow else {
            return Ok(());
        };

        let ((follower, follower_user), followed_inbox) = ctx
            .state
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    let follower_info_fut = accounts::table
                        .find(follow.follower_id)
                        .inner_join(users::table)
                        .select(<(Account, User)>::as_select())
                        .get_result::<(Account, User)>(db_conn);

                    let followed_inbox_fut = accounts::table
                        .find(follow.account_id)
                        .select(accounts::inbox_url)
                        .get_result::<Option<String>>(db_conn);

                    try_join!(follower_info_fut, followed_inbox_fut)
                }
                .scoped()
            })
            .await?;

        if let Some(followed_inbox) = followed_inbox {
            let follow_activity = follow.into_activity(&ctx.state).await?;

            ctx.deliverer
                .deliver(&followed_inbox, &follower, &follower_user, &follow_activity)
                .await?;
        }

        Ok(())
    }
}
