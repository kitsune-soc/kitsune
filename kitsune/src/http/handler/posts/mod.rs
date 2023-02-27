use crate::{
    error::{ApiError, Result},
    mapping::IntoObject,
    service::post::PostService,
    state::Zustand,
};
use askama::Template;
use axum::{debug_handler, extract::Path, extract::State, routing, Json, Router};
use futures_util::future::OptionFuture;
use kitsune_db::entity::prelude::{Accounts, MediaAttachments};
use kitsune_type::ap::Object;
use sea_orm::EntityTrait;
use uuid::Uuid;

mod activity;

#[derive(Template)]
#[template(path = "pages/post.html", escape = "none")] // Make sure everything is escaped either on submission or in the template
struct PostPage {
    display_name: String,
    acct: String,
    profile_url: String,
    profile_picture_url: String,
    content: String,
    version: &'static str,
}

#[debug_handler(state = Zustand)]
async fn get(
    State(state): State<Zustand>,
    State(post): State<PostService>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>> {
    let post = post.get_by_id(id, None).await?.ok_or(ApiError::NotFound)?;

    Ok(Json(post.into_object(&state).await?))
}

#[debug_handler(state = Zustand)]
async fn get_html(
    State(state): State<Zustand>,
    State(post): State<PostService>,
    Path(id): Path<Uuid>,
) -> Result<PostPage> {
    let post = post.get_by_id(id, None).await?.ok_or(ApiError::NotFound)?;
    let author = Accounts::find_by_id(post.account_id)
        .one(&state.db_conn)
        .await?
        .expect("[Bug] Post without author");

    let profile_picture_url = OptionFuture::from(
        author
            .avatar_id
            .map(|id| MediaAttachments::find_by_id(id).one(&state.db_conn)),
    )
    .await
    .transpose()?
    .flatten()
    .map(|attachment| attachment.url);

    let mut acct = format!("@{}", author.username);
    if let Some(domain) = author.domain {
        acct.push('@');
        acct.push_str(&domain);
    }

    Ok(PostPage {
        display_name: author
            .display_name
            .unwrap_or_else(|| author.username.clone()),
        acct,
        profile_url: author.url,
        profile_picture_url: profile_picture_url
            .unwrap_or_else(|| "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()),
        content: post.content,
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get_html))
        .route("/:id/activity", routing::get(activity::get))
}
