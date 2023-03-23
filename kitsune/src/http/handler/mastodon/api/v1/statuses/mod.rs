use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::post::{CreatePost, DeletePost, PostService},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use kitsune_type::mastodon::status::Visibility;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod context;
pub mod favourite;
pub mod unfavourite;

#[derive(Deserialize, ToSchema)]
pub struct CreateForm {
    #[serde(default)]
    media_ids: Vec<Uuid>,
    status: String,
    in_reply_to_id: Option<Uuid>,
    #[serde(default)]
    sensitive: bool,
    spoiler_text: Option<String>,
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    delete,
    path = "/api/v1/statuses/{id}",
    responses(
        (status = StatusCode::OK, description = "Status was deleted"),
    )
)]
async fn delete(
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let delete_post = DeletePost::builder()
        .account_id(user_data.account.id)
        .user_id(user_data.user.id)
        .post_id(id)
        .build()
        .unwrap();

    post.delete(delete_post).await?;

    Ok(StatusCode::OK.into_response())
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/statuses/{id}",
    responses(
        (status = 200, description = "The requested status", body = Status),
        (status = 404, description = "Requested status doesn't exist"),
    )
)]
async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    user_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let account_id = user_data.map(|user_data| user_data.0.account.id);
    let Some(post) = post.get_by_id(id, account_id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(mastodon_mapper.map(post).await?).into_response())
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/statuses",
    request_body = CreateForm,
    responses(
        (status = 200, description = "Newly created post", body = Status),
    )
)]
async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    FormOrJson(form): FormOrJson<CreateForm>,
) -> Result<Response> {
    let mut create_post = CreatePost::builder()
        .author_id(user_data.account.id)
        .content(form.status)
        .media_ids(form.media_ids)
        .sensitive(form.sensitive)
        .visibility(form.visibility.into())
        .clone();

    if let Some(subject) = form.spoiler_text {
        create_post.subject(subject);
    }
    if let Some(in_reply_to_id) = form.in_reply_to_id {
        create_post.in_reply_to_id(in_reply_to_id);
    }

    let status = mastodon_mapper
        .map(post.create(create_post.build().unwrap()).await?)
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
