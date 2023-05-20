use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
};
use async_trait::async_trait;
use kitsune_db::{
    model::favourite::Favourite,
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
        let favourite = posts_favourites::table
            .find(self.favourite_id)
            .first::<Favourite>(&ctx.state.db_conn)
            .await?;

        let (account, user) = accounts::table
            .filter(accounts::id.eq(favourite.account_id))
            .inner_join(users::table.on(users::account_id.eq(accounts::id)))
            .first(&ctx.state.db_conn)
            .await?;

        let inbox_url = posts::table
            .find(favourite.post_id)
            .inner_join(accounts::table)
            .select(accounts::inbox_url)
            .first(&self.db_conn)
            .await?;

        let activity = favourite.into_activity(ctx.state).await?;

        ctx.deliverer
            .deliver(&inbox_url, &account, &user, &activity)
            .await?;

        Ok(())
    }
}
