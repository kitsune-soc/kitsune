use crate::{http::responder::ActivityPubJson, state::Zustand};
use axum::{debug_handler, extract::Path, extract::State};
use kitsune_activitypub::mapping::IntoObject;
use kitsune_error::Result;
use kitsune_service::custom_emoji::CustomEmojiService;
use kitsune_type::ap::emoji::Emoji;
use speedy_uuid::Uuid;

#[debug_handler(state = Zustand)]
pub async fn get(
    State(state): State<Zustand>,
    State(emoji_service): State<CustomEmojiService>,
    Path(id): Path<Uuid>,
) -> Result<ActivityPubJson<Emoji>> {
    let custom_emoji = emoji_service.get_by_id(id).await?;

    Ok(ActivityPubJson(
        custom_emoji.into_object(state.ap_state()).await?,
    ))
}
