use crate::http::graphql::{types::Account, ContextExt};
use async_graphql::{Context, Object, Result};
use kitsune_db::schema::accounts;
use uuid::Uuid;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    pub async fn get_account_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Account> {
        let mut db_conn = ctx.state().db_conn.get().await?;

        Ok(accounts::table
            .find(id)
            .get_result(&mut db_conn)
            .await?
            .map(Into::into))
    }
}
