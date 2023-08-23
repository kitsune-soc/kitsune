use crate::http::graphql::{types::Account, ContextExt};
use async_graphql::{Context, Object, Result};
use speedy_uuid::Uuid;

#[derive(Default)]
pub struct AccountQuery;

#[Object]
impl AccountQuery {
    pub async fn get_account_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Account>> {
        Ok(ctx
            .state()
            .service
            .account
            .get_by_id(id)
            .await?
            .map(Into::into))
    }

    pub async fn my_account(&self, ctx: &Context<'_>) -> Result<Account> {
        let account = &ctx.user_data()?.account;
        Ok(account.clone().into())
    }
}
