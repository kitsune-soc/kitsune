use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{model::favourite::Favourite, schema::posts_favourites, with_connection};
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
        let favourite = with_connection!(ctx.db_pool, |db_conn| {
            posts_favourites::table
                .find(self.favourite_id)
                .get_result::<Favourite>(db_conn)
                .await
                .optional()
        })?;

        let Some(favourite) = favourite else {
            return Ok(());
        };

        ctx.deliverer
            .deliver(Action::Unfavourite(favourite))
            .await?;

        with_connection!(ctx.db_pool, |db_conn| {
            diesel::delete(posts_favourites::table.find(self.favourite_id))
                .execute(db_conn)
                .await
        })?;

        Ok(())
    }
}
