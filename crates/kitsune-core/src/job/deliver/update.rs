use crate::{
    job::{JobRunnerContext, MAX_CONCURRENT_REQUESTS},
    mapping::IntoActivity,
    resolve::InboxResolver,
};
use athena::Runnable;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{StreamExt, TryStreamExt};
use kitsune_db::{
    model::{account::Account, post::Post, user::User},
    schema::{accounts, posts, users},
};
use kitsune_type::ap::ActivityType;
use scoped_futures::ScopedFutureExt;
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

impl Runnable for DeliverUpdate {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let inbox_resolver = InboxResolver::new(ctx.state.db_pool.clone());
        let (activity, account, user, inbox_stream) = match self.entity {
            UpdateEntity::Account => {
                let account_user_data = ctx
                    .state
                    .db_pool
                    .with_connection(|db_conn| {
                        async move {
                            accounts::table
                                .find(self.id)
                                .inner_join(users::table)
                                .select(<(Account, User)>::as_select())
                                .get_result(db_conn)
                                .await
                                .optional()
                        }
                        .scoped()
                    })
                    .await?;

                let Some((account, user)) = account_user_data else {
                    return Ok(());
                };
                let inbox_stream = inbox_resolver.resolve_followers(&account).await?;

                (
                    account.clone().into_activity(&ctx.state).await?,
                    account,
                    user,
                    inbox_stream.left_stream(),
                )
            }
            UpdateEntity::Status => {
                let post_account_user_data = ctx
                    .state
                    .db_pool
                    .with_connection(|db_conn| {
                        async move {
                            posts::table
                                .find(self.id)
                                .inner_join(accounts::table)
                                .inner_join(users::table.on(accounts::id.eq(users::account_id)))
                                .select(<(Post, Account, User)>::as_select())
                                .get_result(db_conn)
                                .await
                                .optional()
                        }
                        .scoped()
                    })
                    .await?;

                let Some((post, account, user)) = post_account_user_data else {
                    return Ok(());
                };

                let inbox_stream = inbox_resolver.resolve(&post).await?;
                let mut activity = post.into_activity(&ctx.state).await?;

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
