use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{model::follower::Follow, schema::accounts_follows};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUnfollow {
    pub follow_id: Uuid,
}

impl Runnable for DeliverUnfollow {
    type Context = JobRunnerContext;
    type Error = miette::Report;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let follow = ctx
            .db_pool
            .with_connection(|db_conn| {
                async move {
                    accounts_follows::table
                        .find(self.follow_id)
                        .get_result::<Follow>(db_conn)
                        .await
                        .optional()
                }
                .scoped()
            })
            .await?;

        let Some(follow) = follow else {
            return Ok(());
        };

        ctx.deliverer
            .deliver(Action::Unfollow(follow))
            .await
            .map_err(|err| miette::Report::new_boxed(err.into()))?;

        ctx.db_pool
            .with_connection(|db_conn| {
                diesel::delete(accounts_follows::table.find(self.follow_id))
                    .execute(db_conn)
                    .scoped()
            })
            .await?;

        Ok(())
    }
}
