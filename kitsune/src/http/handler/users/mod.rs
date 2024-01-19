use crate::{error::Result, http::responder::ActivityPubJson, state::Zustand};
use axum::{
    extract::{Path, State},
    routing, Router,
};
use kitsune_activitypub::mapping::IntoObject;
use kitsune_core::error::HttpError;
use kitsune_service::account::AccountService;
use kitsune_type::ap::actor::Actor;
use speedy_uuid::Uuid;
use tower_http_digest::VerifyDigestLayer;

mod followers;
mod following;
mod inbox;
mod outbox;

async fn get(
    State(state): State<Zustand>,
    State(account_service): State<AccountService>,
    Path(account_id): Path<Uuid>,
) -> Result<ActivityPubJson<Actor>> {
    let account = account_service
        .get_by_id(account_id)
        .await?
        .ok_or(HttpError::NotFound)?;

    Ok(ActivityPubJson(
        account.into_object(state.ap_state()).await?,
    ))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:user_id", routing::get(get))
        .route("/:user_id/followers", routing::get(followers::get))
        .route("/:user_id/following", routing::get(following::get))
        .route(
            "/:user_id/inbox",
            routing::post(inbox::post).layer(VerifyDigestLayer::default()),
        )
        .route("/:user_id/outbox", routing::get(outbox::get))
}
