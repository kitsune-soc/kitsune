use crate::{db::entity::post, http::graphql::ContextExt};
use async_graphql::{Context, Object, Result};
use chrono::Utc;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
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
            ammonia::clean(&buf)
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
}
