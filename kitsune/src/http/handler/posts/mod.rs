use crate::{
    error::Result,
    http::responder::ActivityPubJson,
    state::{PostService, Zustand},
};
use axum::{debug_handler, extract::Path, extract::State, routing, Router};
use kitsune_core::mapping::IntoObject;
use kitsune_type::ap::Object;
use speedy_uuid::Uuid;

mod activity;

#[debug_handler(state = Zustand)]
async fn get(
    State(state): State<Zustand>,
    State(post): State<PostService>,
    Path(id): Path<Uuid>,
) -> Result<ActivityPubJson<Object>> {
    let post = post.get_by_id(id, None).await?;
    Ok(ActivityPubJson(post.into_object(&state.core).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/activity", routing::get(activity::get))
}
