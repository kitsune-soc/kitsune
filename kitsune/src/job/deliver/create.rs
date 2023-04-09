use crate::{
    error::Result,
    job::{JobContext, Runnable, MAX_CONCURRENT_REQUESTS},
    mapping::IntoActivity,
    resolve::InboxResolver,
};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use kitsune_db::entity::prelude::{Accounts, Posts, Users};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverCreate {
    pub post_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverCreate {
    #[instrument(skip_all, fields(post_id = %self.post_id))]
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let Some(post) = Posts::find_by_id(self.post_id)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };

        let (account, user) = Accounts::find_by_id(post.account_id)
            .find_also_related(Users)
            .one(&ctx.state.db_conn)
            .await?
            .expect("[Bug] Post without associated author account");
        let user =
            user.expect("[Bug] Trying to deliver activity for account without associated user");

        let inbox_resolver = InboxResolver::new(ctx.state.db_conn.clone());
        let inbox_stream = inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        let activity = post.into_activity(ctx.state).await?;

        // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
        ctx.deliverer
            .deliver_many(&account, &user, &activity, inbox_stream)
            .await?;

        Ok(())
    }
}
