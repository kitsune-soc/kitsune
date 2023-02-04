use crate::http::graphql::{types::Post, ContextExt};
use async_graphql::{Context, Object, Result};
use kitsune_db::entity::prelude::Posts;
use sea_orm::EntityTrait;
use uuid::Uuid;

#[derive(Default)]
pub struct PostQuery;

#[Object]
impl PostQuery {
    pub async fn get_post_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Post>> {
        Ok(Posts::find_by_id(id)
            .one(&ctx.state().db_conn)
            .await?
            .map(Into::into))
    }
}
