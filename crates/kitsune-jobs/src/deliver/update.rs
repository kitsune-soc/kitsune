use crate::JobRunnerContext;
use athena::Runnable;
use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_core::traits::deliverer::Action;
use kitsune_db::{
    model::{Account, Post},
    schema::{accounts, posts},
    with_connection,
};
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
    type Error = kitsune_error::Error;

    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error> {
        let action = match self.entity {
            UpdateEntity::Account => {
                let account = with_connection!(ctx.db_pool, |db_conn| {
                    accounts::table
                        .find(self.id)
                        .select(Account::as_select())
                        .get_result(db_conn)
                        .await
                        .optional()
                })?;

                let Some(account) = account else {
                    return Ok(());
                };

                Action::UpdateAccount(account)
            }
            UpdateEntity::Status => {
                let post = with_connection!(ctx.db_pool, |db_conn| {
                    posts::table
                        .find(self.id)
                        .select(Post::as_select())
                        .get_result(db_conn)
                        .await
                        .optional()
                })?;

                let Some(post) = post else {
                    return Ok(());
                };

                Action::UpdatePost(post)
            }
        };

        ctx.deliverer.deliver(action).await?;

        Ok(())
    }
}
