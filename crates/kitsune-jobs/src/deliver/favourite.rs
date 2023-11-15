use crate::{error::Error, JobRunnerContext};
use athena::Runnable;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kitsune_core::traits::{deliverer::Action, Deliverer};
use kitsune_db::{model::favourite::Favourite, schema::posts_favourites};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverFavourite {
    pub favourite_id: Uuid,
}

impl Runnable for DeliverFavourite {
    type Context = JobRunnerContext<impl Deliverer>;
    type Error = eyre::Report;

    #[instrument(skip_all, fields(favourite_id = %self.favourite_id))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let favourite = ctx
            .db_pool
            .with_connection(|db_conn| {
                posts_favourites::table
                    .find(self.favourite_id)
                    .get_result::<Favourite>(db_conn)
                    .scoped()
            })
            .await?;

        ctx.deliverer
            .deliver(Action::Favourite(favourite))
            .await
            .map_err(|err| Error::Delivery(err.into()))?;

        Ok(())
    }
}
