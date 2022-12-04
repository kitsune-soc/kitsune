use crate::{db::entity::post, http::graphql::ContextExt, util::CleanHtmlExt};
use async_graphql::{Context, Error, Object, Result};
use chrono::Utc;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter};
use uuid::Uuid;

#[derive(Default)]
pub struct PostMutation;

#[Object]
impl PostMutation {
    pub async fn create_post(&self, ctx: &Context<'_>, content: String) -> Result<post::Model> {
        let state = ctx.state();
        let user = ctx.user()?;
        let content = {
            let parser = Parser::new_ext(&content, Options::all());
            let mut buf = String::new();
            html::push_html(&mut buf, parser);
            buf.clean_html();
            buf
        };

        let id = Uuid::new_v4();
        let url = format!("https://{}/posts/{id}", state.config.domain);
        Ok(post::Model {
            id,
            user_id: user.id,
            subject: None,
            content,
            url,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
        .into_active_model()
        .insert(&state.db_conn)
        .await?)
    }

    pub async fn delete_post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let state = ctx.state();
        let user = ctx.user()?;

        let post = post::Entity::find_by_id(id)
            .belongs_to(user)
            .one(&state.db_conn)
            .await?
            .ok_or_else(|| Error::new("Post not found"))?;

        post.delete(&state.db_conn).await?;

        Ok(id)
    }
}
