use crate::{
    error::{Error, Result},
    http::responder::ActivityPubJson,
    service::url::UrlService,
    state::Zustand,
};
use axum::extract::{OriginalUri, Path, State};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use kitsune_db::schema::{accounts, accounts_follows};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionType},
};
use speedy_uuid::Uuid;

pub async fn get(
    State(state): State<Zustand>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(account_id): Path<Uuid>,
) -> Result<ActivityPubJson<Collection>> {
    let following_count = state
        .db_pool
        .with_connection(|mut db_conn| async move {
            accounts_follows::table
                .inner_join(
                    accounts::table.on(accounts_follows::follower_id
                        .eq(accounts::id)
                        .and(accounts_follows::approved_at.is_not_null())
                        .and(accounts::id.eq(account_id))
                        .and(accounts::local.eq(true))),
                )
                .count()
                .get_result::<i64>(&mut db_conn)
                .await
                .map_err(Error::from)
        })
        .await?;

    let id = format!("{}{}", url_service.base_url(), original_uri.path());
    Ok(ActivityPubJson(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: following_count as u64,
        first: None,
        last: None,
    }))
}
