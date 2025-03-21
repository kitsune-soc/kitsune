use crate::http::extractor::MastodonAuthExtractor;
use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use futures_util::{TryFutureExt, TryStreamExt};
use kitsune_error::{Error, Result};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::post::PostService;
use kitsune_type::mastodon::status::Context;
use speedy_uuid::Uuid;
use std::collections::VecDeque;

#[debug_handler(state = crate::state::Zustand)]
pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    user_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Json<Context>> {
    let account_id = user_data.map(|user_data| user_data.0.account.id);

    let ancestors = post
        .get_ancestors(id, account_id)
        .map_err(Error::from)
        .try_fold(VecDeque::new(), |mut acc, item| async {
            acc.push_front(mastodon_mapper.map(item).await?);
            Ok(acc)
        })
        .await?;

    let descendants = post
        .get_descendants(id, account_id)
        .map_err(Error::from)
        .and_then(|post| mastodon_mapper.map(post).map_err(Error::from))
        .try_collect()
        .await?;

    Ok(Json(Context {
        ancestors,
        descendants,
    }))
}
