use crate::{error::Result, http::responder::ActivityPubJson, state::Zustand};
use axum::{
    extract::{Path, State},
    routing::{self, post},
    Router,
};
use kitsune_core::{error::HttpError, mapping::IntoObject};
use kitsune_service::account::AccountService;
use kitsune_type::ap::actor::Actor;
use speedy_uuid::Uuid;

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

    Ok(ActivityPubJson(account.into_object(&state.core).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:user_id", routing::get(get))
        .route("/:user_id/followers", routing::get(followers::get))
        .route("/:user_id/following", routing::get(following::get))
        .route("/:user_id/inbox", post(inbox::post))
        .route("/:user_id/outbox", routing::get(outbox::get))
}
