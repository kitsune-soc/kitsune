use crate::{
    error::{ApiError, Result},
    http::{
        cond,
        page::{PostComponent, PostPage},
    },
    mapping::IntoObject,
    service::post::PostService,
    state::Zustand,
};
use axum::{debug_handler, extract::Path, extract::State, routing, Json, Router};
use futures_util::TryStreamExt;
use kitsune_type::ap::Object;
use std::collections::VecDeque;
use uuid::Uuid;

mod activity;

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
