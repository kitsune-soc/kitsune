use crate::{
    error::Result, http::extractor::MastodonAuthExtractor, mapping::IntoMastodon, state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::TryStreamExt;
use kitsune_type::mastodon::status::Context;
use std::collections::VecDeque;
use uuid::Uuid;

#[debug_handler]
pub async fn get(
    State(state): State<Zustand>,
    user_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let account_id = user_data.map(|user_data| user_data.0.account.id);

    let ancestors = state
        .service
        .post
        .get_ancestors(id, account_id)
        .try_fold(VecDeque::new(), |mut acc, item| async {
            acc.push_front(item.into_mastodon(&state).await?);
            Ok(acc)
        })
        .await?;

    let descendants = state
        .service
        .post
        .get_descendants(id, account_id)
        .and_then(|post| post.into_mastodon(&state))
        .try_collect()
        .await?;

    Ok(Json(Context {
        ancestors,
        descendants,
    })
    .into_response())
}
