use crate::{
    db::model::{
        job, mention,
        post::{self, Visibility},
    },
    error::Error as ServerError,
    http::graphql::ContextExt,
    job::{deliver_create::CreateDeliveryContext, Job, JobState},
    resolve::MentionResolver,
    sanitize::CleanHtmlExt,
};
use async_graphql::{Context, Error, Object, Result};
use chrono::Utc;
use futures_util::FutureExt;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, TransactionTrait,
};
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
    ) -> Result<post::Model> {
        let state = ctx.state();
        let user = ctx.user_data()?;
        let content = {
            let parser = Parser::new_ext(&content, Options::all());
            let mut buf = String::new();
            html::push_html(&mut buf, parser);
            buf.clean_html();
            buf
        };

        // TODO: Cache this resolver somewhere
        let mention_resolver = MentionResolver::new(
            state.db_conn.clone(),
            state.fetcher.clone(),
            state.webfinger.clone(),
        );
        let (mentioned_account_ids, content) = mention_resolver.resolve(content).await?;

        let id = Uuid::now_v7();
        let account_id = user.account.id;
        let url = format!("https://{}/posts/{id}", state.config.domain);

        state
            .db_conn
            .transaction(move |tx| {
                async move {
                    let post = post::Model {
                        id,
                        account_id,
                        subject: None,
                        content,
                        is_sensitive,
                        visibility,
                        url,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    for account_id in mentioned_account_ids {
                        mention::Model {
                            account_id,
                            post_id: post.id,
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;
                    }

                    let job_context =
                        Job::DeliverCreate(CreateDeliveryContext { post_id: post.id });

                    job::Model {
                        id: Uuid::now_v7(),
                        state: JobState::Queued,
                        run_at: Utc::now(),
                        context: serde_json::to_value(job_context).unwrap(),
                        fail_count: 0,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    Ok::<_, ServerError>(post)
                }
                .boxed()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn delete_post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let state = ctx.state();
        let user = ctx.user_data()?;

        let post = post::Entity::find_by_id(id)
            .belongs_to(&user.account)
            .one(&state.db_conn)
            .await?
            .ok_or_else(|| Error::new("Post not found"))?;

        // TODO: Send out delete activity

        post.delete(&state.db_conn).await?;

        Ok(id)
    }
}
