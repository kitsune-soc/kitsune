use crate::JobRunnerContext;
use athena::Runnable;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{model::Favourite, schema::posts_favourites, with_connection};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverFavourite {
    pub favourite_id: Uuid,
}

impl Runnable for DeliverFavourite {
    type Context = JobRunnerContext;
    type Error = kitsune_error::Error;

    #[cfg_attr(not(coverage), instrument(skip_all, fields(favourite_id = %self.favourite_id)))]
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let favourite = with_connection!(ctx.db_pool, |db_conn| {
            posts_favourites::table
                .find(self.favourite_id)
                .get_result::<Favourite>(db_conn)
                .await
        })?;

        ctx.deliverer.deliver(Action::Favourite(favourite)).await?;

        Ok(())
    }
}
