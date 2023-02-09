use crate::{
    error::Error as ServerError,
    http::graphql::{
        types::{Post, Visibility},
        ContextExt,
    },
    job::{
        deliver::{create::CreateDeliveryContext, delete::DeleteDeliveryContext},
        Job,
    },
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
    search::SearchService,
};
use async_graphql::{Context, Error, Object, Result};
use chrono::Utc;
use futures_util::FutureExt;
use kitsune_db::{
    custom::JobState,
    entity::{jobs, posts, posts_mentions, prelude::Posts},
};
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait,
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
    ) -> Result<Post> {
        let state = ctx.state();
        let mut search_service = state.search_service.clone();
        let user_data = ctx.user_data()?;

        let content = {
            let parser = Parser::new_ext(&content, Options::all());
            let mut buf = String::new();
            html::push_html(&mut buf, parser);
            buf.clean_html();
            buf
        };

        // TODO: Cache this resolver somewhere
        let mention_resolver = PostResolver::new(
            state.db_conn.clone(),
            state.fetcher.clone(),
            state.webfinger.clone(),
        );
        let (mentioned_account_ids, content) = mention_resolver.resolve(&content).await?;

        let id = Uuid::now_v7();
        let account_id = user_data.account.id;
        let url = format!("https://{}/posts/{id}", state.config.domain);

        state
            .db_conn
            .transaction(move |tx| {
                async move {
                    let post = posts::Model {
                        id,
                        account_id,
                        in_reply_to_id: None,
                        subject: None,
                        content,
                        is_sensitive,
                        visibility: visibility.into(),
                        is_local: true,
                        url,
                        created_at: Utc::now().into(),
                        updated_at: Utc::now().into(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    for account_id in mentioned_account_ids {
                        posts_mentions::Model {
                            account_id,
                            post_id: post.id,
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;
                    }

                    let job_context =
                        Job::DeliverCreate(CreateDeliveryContext { post_id: post.id });

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
                    .insert(tx)
                    .await?;

                    if visibility == Visibility::Public || visibility == Visibility::Unlisted {
                        search_service.add_to_index(post.clone()).await?;
                    }

                    Ok::<_, ServerError>(post)
                }
                .boxed()
            })
            .await
            .map(Into::into)
            .map_err(Error::from)
    }

    pub async fn delete_post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Uuid> {
        let state = ctx.state();
        let mut search_service = state.search_service.clone();
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

        search_service.remove_from_index(post).await?;

        Ok(id)
    }
}
