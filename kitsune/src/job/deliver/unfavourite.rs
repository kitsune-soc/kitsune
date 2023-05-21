use crate::{
    error::Result,
    job::{JobContext, Runnable},
    mapping::IntoActivity,
};
use async_trait::async_trait;
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
        let Some(favourite) = PostsFavourites::find_by_id(self.favourite_id)
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

        let favourite_id = favourite.id;
        let activity = favourite.into_negate_activity(ctx.state).await?;

        // TODO: Maybe deliver to followers as well?
        ctx.deliverer
            .deliver(&inbox_url, &account, &user, &activity)
            .await?;

        PostsFavourites::delete_by_id(favourite_id)
            .exec(&ctx.state.db_conn)
            .await?;

        Ok(())
    }
}
