use crate::{
    error::{ApiError, Result},
    http::{
        cond,
        page::{PostComponent, UserPage},
    },
    mapping::IntoObject,
    service::{
        account::{AccountService, GetPosts},
        attachment::AttachmentService,
    },
    state::Zustand,
};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{self, post},
    Json, Router,
};
use futures_util::{future::OptionFuture, TryStreamExt};
use serde::Deserialize;
use uuid::Uuid;

mod followers;
mod following;
mod inbox;
mod outbox;

#[derive(Deserialize)]
struct PageQuery {
    min_id: Option<Uuid>,
    max_id: Option<Uuid>,
}

async fn get_html(
    State(state): State<Zustand>,
    State(account_service): State<AccountService>,
    State(attachment_service): State<AttachmentService>,
    Path(username): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<UserPage> {
    let account = account_service
        .get_local_by_username(&username)
        .await?
        .ok_or(ApiError::NotFound)?;

    let mut get_posts = GetPosts::builder().account_id(account.id).clone();
    if let Some(max_id) = query.max_id {
        get_posts.max_id(max_id);
    }
    if let Some(min_id) = query.min_id {
        get_posts.min_id(min_id);
    }
    let get_posts = get_posts.build().unwrap();

    let posts = account_service
        .get_posts(get_posts)
        .await?
        .and_then(|post| PostComponent::prepare(&state, post))
        .try_collect()
        .await?;

    let mut acct = format!("@{}", account.username);
    if let Some(domain) = account.domain {
        acct.push('@');
        acct.push_str(&domain);
    }

    let profile_picture_url =
        OptionFuture::from(account.avatar_id.map(|id| attachment_service.get_url(id)))
            .await
            .transpose()?;

    Ok(UserPage {
        acct,
        display_name: account.display_name.unwrap_or(account.username),
        profile_picture_url: profile_picture_url
            .unwrap_or_else(|| "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()),
        bio: account.note.unwrap_or_default(),
        posts,
    })
}

async fn get(
    State(state): State<Zustand>,
    State(account_service): State<AccountService>,
    _: State<AttachmentService>, // Needed to get the same types for the conditional routing
    Path(username): Path<String>,
    _: Query<PageQuery>, // Needed to get the same types for the conditional routing
) -> Result<Response> {
    let account = account_service
        .get_local_by_username(&username)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(account.into_object(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:username", routing::get(cond::html(get_html, get)))
        .route("/:username/followers", routing::get(followers::get))
        .route("/:username/following", routing::get(following::get))
        .route("/:username/inbox", post(inbox::post))
        .route("/:username/outbox", routing::get(outbox::get))
}
