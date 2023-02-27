use crate::{
    error::{ApiError, Result},
    http::cond,
    mapping::IntoObject,
    service::post::PostService,
    state::Zustand,
};
use askama::Template;
use axum::{debug_handler, extract::Path, extract::State, routing, Json, Router};
use futures_util::{future::OptionFuture, TryStreamExt};
use kitsune_db::entity::{
    posts,
    prelude::{Accounts, MediaAttachments},
};
use kitsune_type::ap::Object;
use sea_orm::EntityTrait;
use std::collections::VecDeque;
use uuid::Uuid;

mod activity;

#[derive(Template)]
#[template(path = "components/post.html", escape = "none")] // Make sure everything is escaped either on submission or in the template
struct PostComponent {
    display_name: String,
    acct: String,
    profile_url: String,
    profile_picture_url: String,
    content: String,
}

impl PostComponent {
    pub async fn prepare(state: &Zustand, post: posts::Model) -> Result<Self> {
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

        Ok(Self {
            display_name: author
                .display_name
                .unwrap_or_else(|| author.username.clone()),
            acct,
            profile_url: author.url,
            profile_picture_url: profile_picture_url.unwrap_or_else(|| {
                "https://avatarfiles.alphacoders.com/267/thumb-267407.png".into()
            }),
            content: post.content,
        })
    }
}

#[derive(Template)]
#[template(path = "pages/posts.html", escape = "none")]
struct PostPage {
    ancestors: VecDeque<PostComponent>,
    post: PostComponent,
    descendants: Vec<PostComponent>,
    version: &'static str,
}

#[debug_handler(state = Zustand)]
async fn get_html(
    State(state): State<Zustand>,
    State(post_service): State<PostService>,
    Path(id): Path<Uuid>,
) -> Result<PostPage> {
    let post = post_service
        .get_by_id(id, None)
        .await?
        .ok_or(ApiError::NotFound)?;

    let ancestors = post_service
        .get_ancestors(post.id, None)
        .try_fold(VecDeque::new(), |mut acc, item| {
            let state = &state;
            async move {
                let item = PostComponent::prepare(state, item).await?;
                acc.push_front(item);
                Ok(acc)
            }
        })
        .await?;

    let descendants = post_service
        .get_descendants(post.id, None)
        .and_then(|item| PostComponent::prepare(&state, item))
        .try_collect()
        .await?;

    Ok(PostPage {
        ancestors,
        descendants,
        post: PostComponent::prepare(&state, post).await?,
        version: env!("CARGO_PKG_VERSION"),
    })
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

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(cond::html(get_html, get)))
        .route("/:id/activity", routing::get(activity::get))
}
