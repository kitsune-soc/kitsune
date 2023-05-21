use crate::http::graphql::{types::Account, ContextExt};
use async_graphql::{Context, Object, Result};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{model::account::Account as DbAccount, schema::accounts};
use uuid::Uuid;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    pub async fn get_account_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Account> {
        let mut db_conn = ctx.state().db_conn.get().await?;

        Ok(accounts::table
            .find(id)
            .select(DbAccount::as_select())
            .get_result::<DbAccount>(&mut db_conn)
            .await
            .map(Into::into)?)
    }
}
