use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::post::{CreatePost, DeletePost, PostService, UpdatePost},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    routing, Json, Router,
};
use http::StatusCode;
use kitsune_type::mastodon::{status::Visibility, Status};
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::ToSchema;

pub mod context;
pub mod favourite;
pub mod reblog;
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
        .build()
        .unwrap();

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
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    FormOrJson(form): FormOrJson<CreateForm>,
) -> Result<Json<Status>> {
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

    Ok(Json(status))
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
    Path(id): Path<Uuid>,
    FormOrJson(form): FormOrJson<UpdateForm>,
) -> Result<Json<Status>> {
    let update_post = UpdatePost::builder()
        .post_id(id)
        .content(form.status)
        .media_ids(form.media_ids)
        .sensitive(form.sensitive)
        .subject(form.spoiler_text)
        .clone();

    let status = mastodon_mapper
        .map(post.update(update_post.build().unwrap()).await?)
        .await?;

    Ok(Json(status))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::post(post))
        .route("/:id", routing::get(get).delete(delete).put(put))
        .route("/:id/context", routing::get(context::get))
        .route("/:id/favourite", routing::post(favourite::post))
        .route("/:id/reblog", routing::post(reblog::post))
        .route("/:id/unfavourite", routing::post(unfavourite::post))
        .route("/:id/unreblog", routing::post(unreblog::post))
}
