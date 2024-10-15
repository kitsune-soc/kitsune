use crate::http::extractor::{AgnosticForm, AuthExtractor, MastodonAuthExtractor};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use http::StatusCode;
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::post::{CreatePost, DeletePost, PostService, UpdatePost};
use kitsune_type::mastodon::{status::Visibility, Status};
use serde::Deserialize;
use speedy_uuid::Uuid;

pub mod context;
pub mod favourite;
pub mod favourited_by;
pub mod reblog;
pub mod reblogged_by;
pub mod source;
pub mod unfavourite;
pub mod unreblog;

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[debug_handler(state = crate::state::Zustand)]
pub async fn delete(
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

#[debug_handler(state = crate::state::Zustand)]
pub async fn get(
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

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post_service): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    AgnosticForm(form): AgnosticForm<CreateForm>,
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

#[debug_handler(state = crate::state::Zustand)]
pub async fn put(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
    AgnosticForm(form): AgnosticForm<UpdateForm>,
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
