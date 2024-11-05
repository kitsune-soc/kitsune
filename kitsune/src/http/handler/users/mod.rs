use crate::{http::responder::ActivityPubJson, state::Zustand};
use axum::extract::{Path, State};
use kitsune_activitypub::mapping::IntoObject;
use kitsune_error::{kitsune_error, ErrorType, Result};
use kitsune_service::account::AccountService;
use kitsune_type::ap::actor::Actor;
use speedy_uuid::Uuid;

pub mod followers;
pub mod following;
pub mod inbox;
pub mod outbox;

pub async fn get(
    State(state): State<Zustand>,
    State(account_service): State<AccountService>,
    Path(account_id): Path<Uuid>,
) -> Result<ActivityPubJson<Actor>> {
    let account = account_service
        .get_by_id(account_id)
        .await?
        .ok_or_else(|| kitsune_error!(type = ErrorType::NotFound, "account not found"))?;

    Ok(ActivityPubJson(
        account.into_object(state.ap_state()).await?,
    ))
}
