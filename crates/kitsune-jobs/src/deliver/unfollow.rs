use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{model::Follow, schema::accounts_follows, with_connection};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUnfollow {
    pub follow_id: Uuid,
}

impl Runnable for DeliverUnfollow {
    type Context = JobRunnerContext;
    type Error = kitsune_error::Error;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let follow = with_connection!(ctx.db_pool, |db_conn| {
            accounts_follows::table
                .find(self.follow_id)
                .get_result::<Follow>(db_conn)
                .await
                .optional()
        })?;

        let Some(follow) = follow else {
            return Ok(());
        };

        ctx.deliverer.deliver(Action::Unfollow(follow)).await?;

        with_connection!(ctx.db_pool, |db_conn| {
            diesel::delete(accounts_follows::table.find(self.follow_id))
                .execute(db_conn)
                .await
        })?;

        Ok(())
    }
}
