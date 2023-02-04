use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    job::{
        deliver::{create::CreateDeliveryContext, delete::DeleteDeliveryContext},
        Job,
    },
    mapping::IntoMastodon,
    resolve::PostResolver,
    sanitize::CleanHtmlExt,
    search::SearchService,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use chrono::Utc;
use futures_util::FutureExt;
use http::StatusCode;
use kitsune_db::{
    custom::{JobState, Role, Visibility},
    entity::{jobs, posts, posts_mentions, users_roles},
};
use pulldown_cmark::{html, Options, Parser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, TransactionTrait,
};
use serde::Deserialize;
use uuid::Uuid;

mod context;
mod favourite;
mod unfavourite;

#[derive(Deserialize)]
struct CreateForm {
    status: String,
    in_reply_to_id: Option<Uuid>,
    #[serde(default)]
    sensitive: bool,
    spoiler_text: Option<String>,
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = Zustand)]
async fn delete(
    State(mut state): State<Zustand>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = posts::Entity::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if post.account_id != user_data.account.id {
        let admin_role_count = user_data
            .user
            .find_related(users_roles::Entity)
            .filter(users_roles::Column::Role.eq(Role::Administrator))
            .count(&state.db_conn)
            .await?;

        if admin_role_count == 0 {
            return Ok(StatusCode::UNAUTHORIZED.into_response());
        }
    }

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

    state.search_service.remove_from_index(post).await?;

    Ok(StatusCode::OK.into_response())
}

#[debug_handler(state = Zustand)]
async fn get(
    State(state): State<Zustand>,
    _user_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(post) = posts::Entity::find()
        .filter(posts::Column::Id.eq(id))
        .filter(
            posts::Column::Visibility
                .eq(Visibility::Public)
                .or(posts::Column::Visibility.eq(Visibility::Unlisted))
        )
        .one(&state.db_conn)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_mastodon(&state).await?).into_response())
}

#[debug_handler(state = Zustand)]
async fn post(
    State(state): State<Zustand>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    FormOrJson(form): FormOrJson<CreateForm>,
) -> Result<Response> {
    let mut search_service = state.search_service.clone();
    let content = {
        let parser = Parser::new_ext(&form.status, Options::all());
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

    let status = state
        .db_conn
        .transaction(move |tx| {
            async move {
                let in_reply_to_id = if let Some(in_reply_to_id) = form.in_reply_to_id {
                    (posts::Entity::find_by_id(in_reply_to_id).count(tx).await? != 0)
                        .then_some(in_reply_to_id)
                } else {
                    None
                };

                let post = posts::Model {
                    id,
                    account_id,
                    in_reply_to_id,
                    subject: form.spoiler_text,
                    content,
                    is_sensitive: form.sensitive,
                    visibility: form.visibility,
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

                let job_context = Job::DeliverCreate(CreateDeliveryContext { post_id: post.id });

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

                if form.visibility == Visibility::Public || form.visibility == Visibility::Unlisted
                {
                    search_service.add_to_index(post.clone()).await?;
                }

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
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).delete(delete))
        .route("/:id/context", routing::get(context::get))
        .route("/:id/favourite", routing::post(favourite::post))
        .route("/:id/unfavourite", routing::post(unfavourite::post))
}
