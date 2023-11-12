use crate::{job::JobRunnerContext, mapping::IntoActivity};
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, follower::Follow, user::User},
    schema::{accounts, accounts_follows, users},
};
use kitsune_util::try_join;
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUnfollow {
    pub follow_id: Uuid,
}

impl Runnable for DeliverUnfollow {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

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

        let ((follower, follower_user), followed_account_inbox_url, _delete_result) = ctx
            .state
            .db_pool
            .with_connection(|db_conn| {
                async {
                    let follower_info_fut = accounts::table
                        .find(follow.follower_id)
                        .inner_join(users::table)
                        .select(<(Account, User)>::as_select())
                        .get_result::<(Account, User)>(db_conn);

                    let followed_account_inbox_url_fut = accounts::table
                        .find(follow.account_id)
                        .select(accounts::inbox_url)
                        .get_result::<Option<String>>(db_conn);

                    let delete_fut = diesel::delete(&follow).execute(db_conn);

                    try_join!(
                        follower_info_fut,
                        followed_account_inbox_url_fut,
                        delete_fut
                    )
                }
                .scoped()
            })
            .await?;

        if let Some(ref followed_account_inbox_url) = followed_account_inbox_url {
            let follow_activity = follow.into_negate_activity(&ctx.state).await?;

            ctx.deliverer
                .deliver(
                    followed_account_inbox_url,
                    &follower,
                    &follower_user,
                    &follow_activity,
                )
                .await?;
        }

        Ok(())
    }
}
