use crate::{db::model::post, http::graphql::ContextExt};
use async_graphql::{Context, Object, Result};
use sea_orm::EntityTrait;
use uuid::Uuid;

#[derive(Default)]
pub struct PostQuery;

#[Object]
impl PostQuery {
    pub async fn get_post_by_id(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> Result<Option<posts::Model>> {
        Ok(posts::Entity::find_by_id(id)
            .one(&ctx.state().db_conn)
            .await?)
    }
}
