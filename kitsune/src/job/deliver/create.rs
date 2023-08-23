use crate::{
    job::{JobRunnerContext, MAX_CONCURRENT_REQUESTS},
    mapping::IntoActivity,
    resolve::InboxResolver,
};
use async_trait::async_trait;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_db::{
    model::{account::Account, post::Post, user::User},
    schema::{accounts, posts, users},
};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverCreate {
    pub post_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverCreate {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(post_id = %self.post_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let post = ctx
            .state
            .db_pool
            .with_connection(|mut db_conn| async move {
                posts::table
                    .find(self.post_id)
                    .select(Post::as_select())
                    .get_result::<Post>(&mut db_conn)
                    .await
                    .optional()
                    .map_err(Self::Error::from)
            })
            .await?;

        let Some(post) = post else {
            return Ok(());
        };

        let (account, user) = ctx
            .state
            .db_pool
            .with_connection(|mut db_conn| async move {
                accounts::table
                    .find(post.account_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(&mut db_conn)
                    .await
                    .map_err(Self::Error::from)
            })
            .await?;

        let inbox_resolver = InboxResolver::new(ctx.state.db_pool.clone());
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
