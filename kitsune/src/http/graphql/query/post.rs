use crate::http::graphql::{types::Post, ContextExt};
use async_graphql::{Context, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct PostQuery;

#[Object]
impl PostQuery {
    pub async fn get_post_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Post>> {
        let state = ctx.state();
        let account_id = ctx.user_data().ok().map(|user_data| user_data.account.id);

        Ok(state
            .service
            .post
            .get_by_id(id, account_id)
            .await?
            .map(Into::into))
    }
}
