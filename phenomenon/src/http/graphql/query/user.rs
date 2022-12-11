use crate::{db::model::user, http::graphql::ContextExt};
use async_graphql::{Context, Object, Result};
use sea_orm::EntityTrait;
use uuid::Uuid;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    pub async fn get_user_by_id(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<user::Model>> {
        Ok(user::Entity::find_by_id(id)
            .one(&ctx.state().db_conn)
            .await?)
    }
}
