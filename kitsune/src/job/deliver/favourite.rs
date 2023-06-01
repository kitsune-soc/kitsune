use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
    util::assert_future_send,
};
use async_trait::async_trait;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, favourite::Favourite, user::User},
    schema::{accounts, posts, posts_favourites, users},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverFavourite {
    pub favourite_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverFavourite {
    #[instrument(skip_all, fields(favourite_id = %self.favourite_id))]
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let mut db_conn = ctx.state.db_conn.get().await?;
        let favourite = posts_favourites::table
            .find(self.favourite_id)
            .get_result::<Favourite>(&mut db_conn)
            .await?;

        let account_user_fut = assert_future_send(
            accounts::table
                .find(favourite.account_id)
                .inner_join(users::table)
                .select((Account::as_select(), User::as_select()))
                .get_result(&mut db_conn),
        );

        let inbox_url_fut = assert_future_send(
            posts::table
                .find(favourite.post_id)
                .inner_join(accounts::table)
                .select(accounts::inbox_url)
                .get_result::<Option<String>>(&mut db_conn),
        );

        let ((account, user), inbox_url) = tokio::try_join!(account_user_fut, inbox_url_fut)?;

        if let Some(ref inbox_url) = inbox_url {
            let activity = favourite.into_activity(ctx.state).await?;

            ctx.deliverer
                .deliver(inbox_url, &account, &user, &activity)
                .await?;
        }

        Ok(())
    }
}
