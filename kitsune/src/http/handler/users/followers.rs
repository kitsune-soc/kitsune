use crate::{error::Result, http::responder::ActivityPubJson, state::AppState};
use axum::extract::{OriginalUri, Path, State};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_core::service::url::UrlService;
use kitsune_db::schema::{accounts, accounts_follows};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionType},
};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;

pub async fn get(
    State(state): State<AppState>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(account_id): Path<Uuid>,
) -> Result<ActivityPubJson<Collection>> {
    let follower_count = state
        .db_pool
        .with_connection(|db_conn| {
            accounts_follows::table
                .inner_join(
                    accounts::table.on(accounts_follows::account_id
                        .eq(accounts::id)
                        .and(accounts_follows::approved_at.is_not_null())
                        .and(accounts::id.eq(account_id))
                        .and(accounts::local.eq(true))),
                )
                .count()
                .get_result::<i64>(db_conn)
                .scoped()
        })
        .await?;

    let mut id = url_service.base_url();
    id.push_str(original_uri.path());

    Ok(ActivityPubJson(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: follower_count as u64,
        first: None,
        last: None,
    }))
}
