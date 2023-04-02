use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
};
use async_trait::async_trait;
use kitsune_db::{
    column::InboxUrlQuery,
    entity::{
        accounts,
        prelude::{Accounts, Favourites, Users},
    },
    link::FavouritedPostAuthor,
};
use sea_orm::{prelude::*, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverFavourite {
    pub favourite_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverFavourite {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let Some(favourite) = Favourites::find_by_id(self.favourite_id)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };

        let Some((account, Some(user))) = favourite
            .find_related(Accounts)
            .find_also_related(Users)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };

        let inbox_url = favourite
            .find_linked(FavouritedPostAuthor)
            .select_only()
            .column(accounts::Column::InboxUrl)
            .into_values::<String, InboxUrlQuery>()
            .one(&ctx.state.db_conn)
            .await?
            .expect("[Bug] Post without associated account");
        let activity = favourite.into_activity(ctx.state).await?;

        // TODO: Maybe deliver to followers as well?
        ctx.deliverer
            .deliver(&inbox_url, &account, &user, &activity)
            .await?;

        Ok(())
    }
}
