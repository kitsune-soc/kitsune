use crate::{
    error::{ApiError, Result},
    http::responder::ActivityPubJson,
    service::url::UrlService,
    state::Zustand,
};
use axum::extract::{OriginalUri, Path, State};
use kitsune_db::{
    entity::{accounts, prelude::Accounts},
    link::Followers,
};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionType},
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter};
use uuid::Uuid;

pub async fn get(
    State(state): State<Zustand>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(account_id): Path<Uuid>,
) -> Result<ActivityPubJson<Collection>> {
    let Some(account) = Accounts::find_by_id(account_id)
        .filter(accounts::Column::Local.eq(true))
        .one(&state.db_conn)
        .await?
    else {
        return Err(ApiError::NotFound.into());
    };

    let follower_count = account.find_linked(Followers).count(&state.db_conn).await?;

    let mut id = url_service.base_url();
    id.push_str(original_uri.path());

    Ok(ActivityPubJson(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: follower_count,
        first: None,
        last: None,
    }))
}
