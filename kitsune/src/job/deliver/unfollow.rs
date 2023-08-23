use crate::{job::JobRunnerContext, mapping::IntoActivity, try_join};
use async_trait::async_trait;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, follower::Follow, user::User},
    schema::{accounts, accounts_follows, users},
};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUnfollow {
    pub follow_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUnfollow {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let follow = ctx
            .state
            .db_pool
            .with_connection(|mut db_conn| async move {
                accounts_follows::table
                    .find(self.follow_id)
                    .get_result::<Follow>(&mut db_conn)
                    .await
                    .optional()
                    .map_err(Self::Error::from)
            })
            .await?;

        let Some(follow) = follow else {
            return Ok(());
        };

        let ((follower, follower_user), followed_account_inbox_url, _delete_result) = ctx
            .state
            .db_pool
            .with_connection(|mut db_conn| async move {
                let follower_info_fut = accounts::table
                    .find(follow.follower_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(&mut db_conn);

                let followed_account_inbox_url_fut = accounts::table
                    .find(follow.account_id)
                    .select(accounts::inbox_url)
                    .get_result::<Option<String>>(&mut db_conn);

                let delete_fut = diesel::delete(&follow).execute(&mut db_conn);

                try_join!(
                    follower_info_fut,
                    followed_account_inbox_url_fut,
                    delete_fut
                )
                .map_err(Self::Error::from)
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
