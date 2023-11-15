use crate::{error::Error, JobRunnerContext};
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{model::favourite::Favourite, schema::posts_favourites};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUnfavourite {
    pub favourite_id: Uuid,
}

impl Runnable for DeliverUnfavourite {
    type Context = JobRunnerContext;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(favourite_id = %self.favourite_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let favourite = ctx
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    posts_favourites::table
                        .find(self.favourite_id)
                        .get_result::<Favourite>(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        let Some(favourite) = favourite else {
            return Ok(());
        };

        ctx.deliverer
            .deliver(Action::Unfavourite(favourite))
            .await
            .map_err(Error::Delivery)?;

        ctx.db_pool
            .with_connection(|db_conn| {
                diesel::delete(posts_favourites::table.find(self.favourite_id))
                    .execute(db_conn)
                    .scoped()
            })
            .await?;

        Ok(())
    }
}
