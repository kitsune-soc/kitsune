use crate::{
    consts::default_limit,
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{LinkHeader, PaginatedJsonResponse},
    },
};
use axum::{
    Json,
    extract::{OriginalUri, Path, Query, State},
};
use futures_util::{FutureExt, TryFutureExt, TryStreamExt};
use kitsune_error::{Error, Result};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::account::{AccountService, GetPosts};
use kitsune_type::mastodon::Status;
use kitsune_url::UrlService;
use serde::Deserialize;
use speedy_uuid::Uuid;

#[derive(Deserialize)]
pub struct GetQuery {
    max_id: Option<Uuid>,
    since_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

pub async fn get(
    State(account): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    State(url_service): State<UrlService>,
    Path(account_id): Path<Uuid>,
    OriginalUri(original_uri): OriginalUri,
    Query(query): Query<GetQuery>,
    user_data: Option<MastodonAuthExtractor>,
) -> Result<PaginatedJsonResponse<Status>> {
    let fetching_account_id = user_data.as_ref().map(|user_data| user_data.0.account.id);

    let get_posts = GetPosts::builder()
        .account_id(account_id)
        .fetching_account_id(fetching_account_id)
        .max_id(query.max_id)
        .since_id(query.since_id)
        .min_id(query.min_id)
        .limit(query.limit)
        .build();

    let mut statuses: Vec<Status> = account
        .get_posts(get_posts)
        .await?
        .map_err(Error::from)
        .and_then(|post| {
            if let Some(AuthExtractor(ref user_data)) = user_data {
                mastodon_mapper
                    .map((&user_data.account, post))
                    .left_future()
            } else {
                mastodon_mapper.map(post).right_future()
            }
            .map_err(Error::from)
        })
        .try_collect()
        .await?;

    if query.min_id.is_some() {
        statuses.reverse();
    }

    let link_header = LinkHeader::new(
        &statuses,
        query.limit,
        &url_service.base_url(),
        original_uri.path(),
        |s| s.id,
    );

    Ok((link_header, Json(statuses)))
}
