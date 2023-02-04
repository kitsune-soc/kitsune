use crate::{
    error::{Error, Result},
    state::Zustand,
};
use axum::{
    extract::{OriginalUri, Path, State},
    Json,
};
use kitsune_db::{
    entity::{accounts, users},
    link::Following,
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
    let Some(account) = <users::Entity as Related<accounts::Entity>>::find_related()
        .filter(users::Column::Username.eq(username))
        .one(&state.db_conn)
        .await?
    else {
        return Err(Error::UserNotFound);
    };

    let following_count = account.find_linked(Following).count(&state.db_conn).await?;

    let id = format!("https://{}{}", state.config.domain, original_uri.path());
    Ok(Json(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: following_count,
        first: None,
        last: None,
    }))
}
