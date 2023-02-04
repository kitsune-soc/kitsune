use crate::http::graphql::{types::Account, ContextExt};
use async_graphql::{Context, Object, Result};
use kitsune_db::entity::accounts;
use sea_orm::EntityTrait;
use uuid::Uuid;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    pub async fn get_account_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Account>> {
        Ok(accounts::Entity::find_by_id(id)
            .one(&ctx.state().db_conn)
            .await?
            .map(Into::into))
    }
}
