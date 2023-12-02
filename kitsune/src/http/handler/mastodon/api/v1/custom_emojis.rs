use crate::{
    error::{Error, Result},
    http::extractor::MastodonAuthExtractor,
    state::Zustand,
};
use axum::{debug_handler, extract::State, routing, Json, Router};
use futures_util::{TryFutureExt, TryStreamExt};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::custom_emoji::{CustomEmojiService, GetEmojiList};
use kitsune_type::mastodon::CustomEmoji;

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/custom_emojis",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "List of custom emojis available on the server", body = Vec<CustomEmoji>)
    ),
)]
pub async fn get(
    State(custom_emoji_service): State<CustomEmojiService>,
    State(mastodon_mapper): State<MastodonMapper>,
    user_data: Option<MastodonAuthExtractor>,
) -> Result<Json<Vec<CustomEmoji>>> {
    let get_emoji_list = GetEmojiList::builder()
        .fetching_account_id(user_data.map(|x| x.0.account.id))
        .build();

    let custom_emojis: Vec<CustomEmoji> = custom_emoji_service
        .get_list(get_emoji_list)
        .await?
        .map_err(Error::from)
        .and_then(|acc| mastodon_mapper.map(acc).map_err(Error::from))
        .try_collect()
        .await?;

    Ok(Json(custom_emojis))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
