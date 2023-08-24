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
pub struct DeliverDelete {
    pub post_id: Uuid,
}

#[async_trait]
impl Runnable for DeliverDelete {
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
            })
            .await?;

        let Some(post) = post else {
            return Ok(());
        };

        let account_user_data = ctx
            .state
            .db_pool
            .with_connection(|mut db_conn| async move {
                accounts::table
                    .find(post.account_id)
                    .inner_join(users::table)
                    .select(<(Account, User)>::as_select())
                    .get_result::<(Account, User)>(&mut db_conn)
                    .await
                    .optional()
            })
            .await?;

        let Some((account, user)) = account_user_data else {
            return Ok(());
        };

        let inbox_resolver = InboxResolver::new(ctx.state.db_pool.clone());
        let inbox_stream = inbox_resolver
            .resolve(&post)
            .await?
            .try_chunks(MAX_CONCURRENT_REQUESTS)
            .map_err(|err| err.1);

        let post_id = post.id;
        let delete_activity = post.into_negate_activity(&ctx.state).await?;

        // TODO: Should we deliver to the inboxes that are contained inside a `TryChunksError`?
        ctx.deliverer
            .deliver_many(&account, &user, &delete_activity, inbox_stream)
            .await?;

        ctx.state
            .db_pool
            .with_connection(|mut db_conn| {
                diesel::delete(posts::table.find(post_id)).execute(&mut db_conn)
            })
            .await?;

        Ok(())
    }
}
