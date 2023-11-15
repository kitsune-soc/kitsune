use crate::{error::Error, JobRunnerContext};
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use kitsune_core::traits::{deliverer::Action, Deliverer};
use kitsune_db::{model::post::Post, schema::posts};
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

        ctx.deliverer
            .deliver(Action::Create(post))
            .await
            .map_err(|err| Error::Delivery(err.into()))?;

        Ok(())
    }
}
