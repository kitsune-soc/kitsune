use crate::job::{JobContext, JobRunner, MAX_CONCURRENT_REQUESTS};
use crate::{error::Result, mapping::IntoActivity, resolve::InboxResolver};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use kitsune_db::entity::prelude::{Accounts, Posts, Users};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DeliverDelete {
    pub post_id: Uuid,
}

#[async_trait]
impl JobRunner for DeliverDelete {
    async fn run(self, ctx: JobContext<'_>) -> Result<()> {
        let Some(post) = Posts::find_by_id(self.post_id)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };

        let Some((account, Some(user))) = Accounts::find_by_id(post.account_id)
            .find_also_related(Users)
            .one(&ctx.state.db_conn)
            .await?
        else {
            return Ok(());
        };

        let inbox_resolver = InboxResolver::new(ctx.state.db_conn.clone());
        let inbox_stream = inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        Posts::delete_by_id(post.id)
            .exec(&ctx.state.db_conn)
            .await?;

        let delete_activity = post.into_negate_activity(ctx.state).await?;

        // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
        ctx.deliverer
            .deliver_many(&account, &user, &delete_activity, inbox_stream)
            .await?;

        Ok(())
    }
}
