use crate::{
    error::{ApiError, Result},
    state::Zustand,
};
use axum::{
    extract::{OriginalUri, Path, State},
    Json,
};
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
    OriginalUri(original_uri): OriginalUri,
    Path(username): Path<String>,
) -> Result<Json<Collection>> {
    let Some(account) = <Users as Related<Accounts>>::find_related()
        .filter(users::Column::Username.eq(username))
        .one(&state.db_conn)
        .await?
    else {
        return Err(ApiError::NotFound.into());
    };

    let follower_count = account.find_linked(Followers).count(&state.db_conn).await?;

    let id = format!("https://{}{}", state.config.domain, original_uri.path());
    Ok(Json(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: follower_count,
        first: None,
        last: None,
    }))
}
