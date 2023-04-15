use crate::{
    error::Result,
    job::{JobContext, Runnable, MAX_CONCURRENT_REQUESTS},
    mapping::IntoActivity,
    resolve::InboxResolver,
};
use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use kitsune_db::entity::prelude::{Accounts, Posts, Users};
use kitsune_type::ap::ActivityType;
use sea_orm::{EntityTrait, ModelTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub enum UpdateEntity {
    Account,
    Status,
}

#[derive(Deserialize, Serialize)]
pub struct DeliverUpdate {
    pub entity: UpdateEntity,
    pub id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUpdate {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        let inbox_resolver = InboxResolver::new(ctx.state.db_conn.clone());
        let (activity, account, user, inbox_stream) = match self.entity {
            UpdateEntity::Account => {
                let Some((account, Some(user))) = Accounts::find_by_id(self.id)
                    .find_also_related(Users)
                    .one(&ctx.state.db_conn)
                    .await?
                else {
                    return Ok(());
                };

                let inbox_stream = inbox_resolver.resolve_followers(&account).await?;

                (
                    account.clone().into_activity(ctx.state).await?,
                    account,
                    user,
                    inbox_stream.left_stream(),
                )
            }
            UpdateEntity::Status => {
                let Some((post, Some(account))) = Posts::find_by_id(self.id)
                    .find_also_related(Accounts)
                    .one(&ctx.state.db_conn)
                    .await?
                else {
                    return Ok(());
                };
                let Some(user) = account.find_related(Users).one(&ctx.state.db_conn).await? else {
                    error!("tried to update non-local post");
                    return Ok(());
                };

                let inbox_stream = inbox_resolver.resolve(&post).await?;
                let mut activity = post.into_activity(ctx.state).await?;

                // Patch in the update
                activity.r#type = ActivityType::Update;

                (activity, account, user, inbox_stream.right_stream())
            }
        };

        let inbox_stream = inbox_stream
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        ctx.deliverer
            .deliver_many(&account, &user, &activity, inbox_stream)
            .await?;

        Ok(())
    }
}
