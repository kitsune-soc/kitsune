use crate::job::{JobContext, MAX_CONCURRENT_REQUESTS};
use async_trait::async_trait;
use athena::Runnable;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{StreamExt, TryStreamExt};
use kitsune_db::{
    model::{account::Account, post::Post, user::User},
    schema::{accounts, posts, users},
};
use kitsune_type::ap::ActivityType;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub enum UpdateEntity {
    Account,
    Status,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUpdate {
    pub entity: UpdateEntity,
    pub id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUpdate {
    type Context = JobContext;
    type Error = anyhow::Error;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let inbox_resolver = InboxResolver::new(ctx.db_conn.clone());
        let mut db_conn = ctx.state.db_conn.get().await?;
        let (activity, account, user, inbox_stream) = match self.entity {
            UpdateEntity::Account => {
                let Some((account, user)) = accounts::table
                    .find(self.id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result(&mut db_conn)
                    .await
                    .optional()?
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
                let Some((post, account, user)) = posts::table
                    .find(self.id)
                    .inner_join(accounts::table)
                    .inner_join(users::table.on(accounts::id.eq(users::account_id)))
                    .select(<(Post, Account, User)>::as_select())
                    .get_result(&mut db_conn)
                    .await
                    .optional()?
                else {
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
