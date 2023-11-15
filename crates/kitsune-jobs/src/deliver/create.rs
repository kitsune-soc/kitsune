use crate::{
    mapping::IntoActivity, resolve::InboxResolver, JobRunnerContext, MAX_CONCURRENT_REQUESTS,
};
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_core::traits::Deliverer;
use kitsune_db::{
    model::{account::Account, post::Post, user::User},
    schema::{accounts, posts, users},
};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverCreate {
    pub post_id: Uuid,
}

impl Runnable for DeliverCreate {
    type Context = JobRunnerContext<impl Deliverer>;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(post_id = %self.post_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let post = ctx
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    posts::table
                        .find(self.post_id)
                        .select(Post::as_select())
                        .get_result::<Post>(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        let Some(post) = post else {
            return Ok(());
        };

        let (account, user) = ctx
            .db_pool
            .with_connection(|db_conn| {
                accounts::table
                    .find(post.account_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(db_conn)
                    .scoped()
            })
            .await?;

        let inbox_resolver = InboxResolver::new(ctx.db_pool.clone());
        let inbox_stream = inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        let activity = post.into_activity(&ctx.state).await?;

        // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
        ctx.deliverer
            .deliver_many(&account, &user, &activity, inbox_stream)
            .await?;

        Ok(())
    }
}
