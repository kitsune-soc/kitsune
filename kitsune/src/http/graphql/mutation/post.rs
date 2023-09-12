use crate::http::graphql::{
    types::{Post, Visibility},
    ContextExt,
};
use async_graphql::{Context, Object, Result};
use kitsune_core::service::post::{CreatePost, DeletePost};
use speedy_uuid::Uuid;

#[derive(Default)]
pub struct PostMutation;

#[Object]
impl PostMutation {
    pub async fn create_post(
        &self,
        ctx: &Context<'_>,
        content: String,
        is_sensitive: bool,
        visibility: Visibility,
    ) -> Result<Post> {
        let state = ctx.state();
        let user_data = ctx.user_data()?;

        let create_post = CreatePost::builder()
            .author_id(user_data.account.id)
            .sensitive(is_sensitive)
            .content(content)
            .visibility(visibility.into())
            .build()
            .unwrap();

        let post = state.service().post.create(create_post).await?;

        Ok(post.into())
    }

    pub async fn delete_post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let state = ctx.state();
        let user_data = ctx.user_data()?;

        let delete_post = DeletePost::builder()
            .account_id(user_data.account.id)
            .user_id(user_data.user.id)
            .post_id(id)
            .build()
            .unwrap();

        state.service().post.delete(delete_post).await?;

        Ok(id)
    }
}
