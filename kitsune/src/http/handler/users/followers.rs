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
    link::Followers,
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
