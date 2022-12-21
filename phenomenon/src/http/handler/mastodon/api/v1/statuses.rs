use crate::{
    db::model::{
        job, mention,
        post::{self, Visibility},
    },
    error::Result,
    http::extractor::{AuthExtactor, FormOrJson},
    job::{deliver_create::CreateDeliveryContext, Job, JobState},
    mapping::IntoMastodon,
    resolve::MentionResolver,
    sanitize::CleanHtmlExt,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use futures_util::FutureExt;
use http::StatusCode;
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionTrait,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct CreateForm {
    status: String,
    #[serde(default)]
    sensitive: bool,
    spoiler_text: Option<String>,
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = Zustand)]
async fn get_handler(
    State(state): State<Zustand>,
    AuthExtactor(_account): AuthExtactor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = post::Entity::find()
        .filter(post::Column::Id.eq(id))
        .filter(
            post::Column::Visibility
                .eq(Visibility::Public)
                .or(post::Column::Visibility.eq(Visibility::Unlisted))
        )
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_mastodon(&state).await?).into_response())
}

#[debug_handler(state = Zustand)]
async fn post_handler(
    State(state): State<Zustand>,
    AuthExtactor(user_data): AuthExtactor,
    FormOrJson(form): FormOrJson<CreateForm>,
) -> Result<Response> {
    let Some(user_data) = user_data else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    let content = {
        let parser = Parser::new_ext(&form.status, Options::all());
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
    let account_id = user_data.account.id;
    let url = format!("https://{}/posts/{id}", state.config.domain);

    let status = state
        .db_conn
        .transaction(move |tx| {
            async move {
                let post = post::Model {
                    id,
                    account_id,
                    subject: form.spoiler_text,
                    content,
                    is_sensitive: form.sensitive,
                    visibility: form.visibility,
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

                let job_context = Job::DeliverCreate(CreateDeliveryContext { post_id: post.id });

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

                Ok(post)
            }
            .boxed()
        })
        .await?
        .into_mastodon(&state)
        .await?;

    Ok(Json(status).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", post(post_handler))
        .route("/:id", get(get_handler))
}
