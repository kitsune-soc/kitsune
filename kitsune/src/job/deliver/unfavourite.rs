use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
};
use async_trait::async_trait;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{account::Account, favourite::Favourite, user::User},
    schema::{accounts, posts, posts_favourites, users},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverUnfavourite {
    pub favourite_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUnfavourite {
    #[instrument(skip_all, fields(favourite_id = %self.favourite_id))]
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let mut db_conn = ctx.state.db_conn.get().await?;
        let Some(favourite) = posts_favourites::table
            .find(self.favourite_id)
            .get_result::<Favourite>(&mut db_conn)
            .await
            .optional()?
        else {
            return Ok(());
        };

        let (account, user) = accounts::table
            .find(favourite.account_id)
            .inner_join(users::table)
            .select((Account::as_select(), User::as_select()))
            .get_result(&mut db_conn)
            .await?;

        let inbox_url = posts::table
            .find(favourite.post_id)
            .inner_join(accounts::table)
            .select(accounts::inbox_url)
            .get_result::<Option<String>>(&mut db_conn)
            .await?;

        let favourite_id = favourite.id;
        if let Some(ref inbox_url) = inbox_url {
            let activity = favourite.into_negate_activity(ctx.state).await?;
            ctx.deliverer
                .deliver(inbox_url, &account, &user, &activity)
                .await?;
        }

        diesel::delete(posts_favourites::table.find(favourite_id))
            .execute(&mut db_conn)
            .await?;

        Ok(())
    }
}
