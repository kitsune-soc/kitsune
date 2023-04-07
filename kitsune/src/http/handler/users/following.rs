use crate::{
    error::{ApiError, Result},
    http::responder::ActivityPubJson,
    service::url::UrlService,
    state::Zustand,
};
use axum::extract::{OriginalUri, Path, State};
use kitsune_db::{
    entity::{
        prelude::{Accounts, Users},
        users,
    },
    link::Following,
};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionType},
};
use sea_orm::{ColumnTrait, ModelTrait, PaginatorTrait, QueryFilter, Related};

pub async fn get(
    State(state): State<Zustand>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(username): Path<String>,
) -> Result<ActivityPubJson<Collection>> {
    let Some(account) = <Users as Related<Accounts>>::find_related()
        .filter(users::Column::Username.eq(username))
        .one(&state.db_conn)
        .await?
    else {
        return Err(ApiError::NotFound.into());
    };

    let following_count = account.find_linked(Following).count(&state.db_conn).await?;

    let id = format!("{}{}", url_service.base_url(), original_uri.path());
    Ok(ActivityPubJson(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: following_count,
        first: None,
        last: None,
    }))
}
