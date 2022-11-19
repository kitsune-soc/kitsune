use crate::{db::entity::token, http::graphql::ContextExt};
use async_graphql::{Context, Object, Result};
use sea_orm::EntityTrait;

#[derive(Default)]
pub struct AuthQuery;

#[Object]
impl AuthQuery {
    pub async fn introspect_token(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<Option<token::Model>> {
        Ok(token::Entity::find_by_id(token)
            .one(&ctx.state().db_conn)
            .await?)
    }
}
