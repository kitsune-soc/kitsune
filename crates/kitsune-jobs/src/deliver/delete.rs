use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{model::post::Post, schema::posts, with_connection};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverDelete {
    pub post_id: Uuid,
}

impl Runnable for DeliverDelete {
    type Context = JobRunnerContext;
    type Error = kitsune_error::Error;

    #[cfg_attr(not(coverage), instrument(skip_all, fields(post_id = %self.post_id)))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let post = with_connection!(ctx.db_pool, |db_conn| {
            posts::table
                .find(self.post_id)
                .select(Post::as_select())
                .get_result::<Post>(db_conn)
                .await
                .optional()
        })?;

        let Some(post) = post else {
            return Ok(());
        };

        ctx.deliverer.deliver(Action::Delete(post)).await?;

        with_connection!(ctx.db_pool, |db_conn| {
            diesel::delete(posts::table.find(self.post_id))
                .execute(db_conn)
                .await
        })?;

        Ok(())
    }
}
