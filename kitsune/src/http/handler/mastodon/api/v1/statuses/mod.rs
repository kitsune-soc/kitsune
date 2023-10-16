use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    routing, Json, Router,
};
use http::StatusCode;
use kitsune_core::{
    mapping::MastodonMapper,
    service::post::{CreatePost, DeletePost, PostService, UpdatePost},
};
use kitsune_type::mastodon::{status::Visibility, Status};
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::ToSchema;

pub mod context;
pub mod favourite;
pub mod favourited_by;
pub mod reblog;
pub mod reblogged_by;
pub mod source;
pub mod unfavourite;
pub mod unreblog;

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

#[derive(Deserialize, ToSchema)]
pub struct UpdateForm {
    #[serde(default)]
    media_ids: Vec<Uuid>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    sensitive: Option<bool>,
    #[serde(default)]
    spoiler_text: Option<String>,
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    delete,
    path = "/api/v1/statuses/{id}",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "Status was deleted"),
    )
)]
async fn delete(
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let delete_post = DeletePost::builder()
        .account_id(user_data.account.id)
        .user_id(user_data.user.id)
        .post_id(id)
        .build();

    post.delete(delete_post).await?;

    Ok(StatusCode::OK)
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/statuses/{id}",
    security(
        (),
        ("oauth_token" = [])
    ),
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
) -> Result<Json<Status>> {
    let account_id = user_data.as_ref().map(|user_data| user_data.0.account.id);
    let post = post.get_by_id(id, account_id).await?;

    let status = if let Some(AuthExtractor(ref user_data)) = user_data {
        mastodon_mapper.map((&user_data.account, post)).await?
    } else {
        mastodon_mapper.map(post).await?
    };

    Ok(Json(status))
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/statuses",
    security(
        ("oauth_token" = [])
    ),
    request_body = CreateForm,
    responses(
        (status = 200, description = "Newly created post", body = Status),
    )
)]
async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post_service): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    FormOrJson(form): FormOrJson<CreateForm>,
) -> Result<Json<Status>> {
    let create_post = CreatePost::builder()
        .author_id(user_data.account.id)
        .content(form.status)
        .in_reply_to_id(form.in_reply_to_id)
        .media_ids(form.media_ids)
        .sensitive(form.sensitive)
        .subject(form.spoiler_text.filter(|subject| !subject.is_empty()))
        .visibility(form.visibility.into())
        .build();

    let post = post_service.create(create_post).await?;

    Ok(Json(mastodon_mapper.map(post).await?))
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    put,
    path = "/api/v1/statuses/{id}",
    security(
        ("oauth_token" = [])
    ),
    request_body = UpdateForm,
    responses(
        (status = StatusCode::OK, description = "Status has been successfully edited", body = Status),
        (status = StatusCode::NOT_FOUND, description = "Requested status doesn't exist"),
    )
)]
async fn put(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
    FormOrJson(form): FormOrJson<UpdateForm>,
) -> Result<Json<Status>> {
    let update_post = UpdatePost::builder()
        .account_id(user_data.account.id)
        .post_id(id)
        .content(form.status)
        .media_ids(form.media_ids)
        .sensitive(form.sensitive)
        .subject(form.spoiler_text.filter(|subject| !subject.is_empty()))
        .build();

    let status = mastodon_mapper.map(post.update(update_post).await?).await?;

    Ok(Json(status))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).delete(delete).put(put))
        .route("/:id/context", routing::get(context::get))
        .route("/:id/favourite", routing::post(favourite::post))
        .route("/:id/favourited_by", routing::get(favourited_by::get))
        .route("/:id/reblog", routing::post(reblog::post))
        .route("/:id/reblogged_by", routing::get(reblogged_by::get))
        .route("/:id/source", routing::get(source::get))
        .route("/:id/unfavourite", routing::post(unfavourite::post))
        .route("/:id/unreblog", routing::post(unreblog::post))
}
