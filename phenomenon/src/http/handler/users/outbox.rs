use crate::{
    db::model::{
        account, follow,
        post::{self, Visibility},
        user,
    },
    error::{Error, Result},
    state::Zustand,
};
use axum::{
    extract::{Path, State},
    Json,
};
use phenomenon_type::ap::{
    ap_context,
    collection::{Collection, CollectionType},
};
use sea_orm::{ColumnTrait, ModelTrait, PaginatorTrait, QueryFilter, Related};

pub async fn get(
    State(state): State<Zustand>,
    Path(username): Path<String>,
) -> Result<Json<Collection>> {
    let Some(account) = <user::Entity as Related<account::Entity>>::find_related()
        .filter(user::Column::Username.eq(username))
        .one(&state.db_conn)
        .await?
    else {
        return Err(Error::UserNotFound);
    };

    let public_post_count = account
        .find_related(post::Entity)
        .filter(post::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]))
        .count(&state.db_conn)
        .await?;

    let id = format!(
        "https://{}/users/{}/outbox",
        state.config.domain, account.username
    );
    Ok(Json(Collection {
        context: ap_context(),
        id,
        r#type: CollectionType::OrderedCollection,
        total_items: public_post_count,
        first: None,
        last: None,
    }))
}
