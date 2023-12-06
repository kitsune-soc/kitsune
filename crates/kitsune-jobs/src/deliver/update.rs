use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{
    model::{account::Account, post::Post},
    schema::{accounts, posts},
};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub enum UpdateEntity {
    Account,
    Status,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeliverUpdate {
    pub entity: UpdateEntity,
    pub id: Uuid,
}

impl Runnable for DeliverUpdate {
    type Context = JobRunnerContext;
    type Error = miette::Report;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let action = match self.entity {
            UpdateEntity::Account => {
                let account = ctx
                    .db_pool
                    .with_connection(|db_conn| {
                        async move {
                            accounts::table
                                .find(self.id)
                                .select(Account::as_select())
                                .get_result(db_conn)
                                .await
                                .optional()
                        }
                        .scoped()
                    })
                    .await?;

                let Some(account) = account else {
                    return Ok(());
                };

                Action::UpdateAccount(account)
            }
            UpdateEntity::Status => {
                let post = ctx
                    .db_pool
                    .with_connection(|db_conn| {
                        async move {
                            posts::table
                                .find(self.id)
                                .select(Post::as_select())
                                .get_result(db_conn)
                                .await
                                .optional()
                        }
                        .scoped()
                    })
                    .await?;

                let Some(post) = post else {
                    return Ok(());
                };

                Action::UpdatePost(post)
            }
        };

        ctx.deliverer
            .deliver(action)
            .await
            .map_err(|err| miette::Report::new_boxed(err.into()))?;

        Ok(())
    }
}
