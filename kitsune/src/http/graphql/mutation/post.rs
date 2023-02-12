use crate::{
    http::graphql::{
        types::{Post, Visibility},
        ContextExt,
    },
    job::{deliver::delete::DeleteDeliveryContext, Job},
    service::{post::CreatePost, search::SearchService},
};
use async_graphql::{Context, Error, Object, Result};
use chrono::Utc;
use kitsune_db::{
    custom::JobState,
    entity::{jobs, posts, prelude::Posts},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use uuid::Uuid;

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

        let post = state.service.post.create(create_post).await?;

        Ok(post.into())
    }

    pub async fn delete_post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let state = ctx.state();
        let user_data = ctx.user_data()?;

        let post = Posts::find_by_id(id)
            .filter(posts::Column::AccountId.eq(user_data.account.id))
            .one(&state.db_conn)
            .await?
            .ok_or_else(|| Error::new("Post not found"))?;

        let job_context = Job::DeliverDelete(DeleteDeliveryContext { post_id: post.id });
        jobs::Model {
            id: Uuid::now_v7(),
            state: JobState::Queued,
            run_at: Utc::now().into(),
            context: serde_json::to_value(job_context).unwrap(),
            fail_count: 0,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&state.db_conn)
        .await?;
        state.service.search.remove_from_index(post).await?;

        Ok(id)
    }
}
