use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
};
use async_trait::async_trait;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, follower::Follow, user::User},
    schema::{accounts, accounts_follows, users},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverUnfollow {
    pub follow_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUnfollow {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let mut db_conn = ctx.state.db_conn.get().await?;
        let Some(follow) = accounts_follows::table
            .find(self.follow_id)
            .get_result::<Follow>(&mut db_conn)
            .await
            .optional()?
        else {
            return Ok(());
        };

        let follower_data_fut = accounts::table
            .find(follow.follower_id)
            .inner_join(users::table)
            .select((Account::as_select(), User::as_select()))
            .get_result::<(Account, User)>(&mut db_conn);

        let followed_account_inbox_url_fut = accounts::table
            .find(follow.account_id)
            .select(accounts::inbox_url)
            .get_result::<Option<String>>(&mut db_conn);

        let delete_follow_fut = diesel::delete(&follow).execute(&mut db_conn);

        let ((follower, follower_user), followed_account_inbox_url, _delete_follow_result) = tokio::try_join!(
            follower_data_fut,
            followed_account_inbox_url_fut,
            delete_follow_fut,
        )?;

        if let Some(ref followed_account_inbox_url) = followed_account_inbox_url {
            let follow_activity = follow.into_negate_activity(ctx.state).await?;

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
