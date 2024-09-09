use crate::{http::responder::ActivityPubJson, state::Zustand};
use axum::{debug_handler, extract::Path, extract::State};
use kitsune_activitypub::mapping::IntoObject;
use kitsune_error::Result;
use kitsune_service::post::PostService;
use kitsune_type::ap::Object;
use speedy_uuid::Uuid;

pub mod activity;

#[debug_handler(state = Zustand)]
pub async fn get(
    State(state): State<Zustand>,
    State(post): State<PostService>,
    Path(id): Path<Uuid>,
) -> Result<ActivityPubJson<Object>> {
    let post = post.get_by_id(id, None).await?;
    Ok(ActivityPubJson(post.into_object(state.ap_state()).await?))
}
