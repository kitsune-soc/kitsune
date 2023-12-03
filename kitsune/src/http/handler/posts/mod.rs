use crate::{error::Result, http::responder::ActivityPubJson, state::Zustand};
use axum::{debug_handler, extract::Path, extract::State, routing, Router};
use kitsune_activitypub::mapping::IntoObject;
use kitsune_service::post::PostService;
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
    Ok(ActivityPubJson(post.into_object(state.ap_state()).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/activity", routing::get(activity::get))
}
