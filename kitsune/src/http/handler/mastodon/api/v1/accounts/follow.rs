use crate::http::extractor::{AgnosticForm, AuthExtractor, MastodonAuthExtractor};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_error::{bail, ErrorType, Result};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::account::{AccountService, Follow};
use kitsune_type::mastodon::relationship::Relationship;
use serde::Deserialize;
use speedy_uuid::Uuid;

#[derive(Deserialize)]
pub struct FollowBody {
    #[serde(default)]
    notify: bool,
}

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
    follow_body: Option<AgnosticForm<FollowBody>>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        bail!(type = ErrorType::BadRequest, "user tried to follow themselves");
    }

    let follow = Follow::builder()
        .account_id(id)
        .follower_id(user_data.account.id)
        .build();
    let follow_accounts = account_service
        .follow(follow, follow_body.map_or(false, |body| body.0.notify))
        .await?;

    Ok(Json(
        mastodon_mapper
            .map((&follow_accounts.0, &follow_accounts.1))
            .await?,
    ))
}
