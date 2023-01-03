use crate::{
    db::model::{account, follow, user},
    error::{Error, Result},
    state::Zustand,
};
use axum::{
    extract::{OriginalUri, Path, State},
    Json,
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
    let Some(account) = <user::Entity as Related<account::Entity>>::find_related()
        .filter(user::Column::Username.eq(username))
        .one(&state.db_conn)
        .await?
    else {
        return Err(Error::UserNotFound);
    };

    let follower_count = account
        .find_linked(follow::Followers)
        .count(&state.db_conn)
        .await?;

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
