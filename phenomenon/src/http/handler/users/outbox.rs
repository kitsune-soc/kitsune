use crate::{
    db::model::{
        account,
        post::{self, Visibility},
        user,
    },
    error::{Error, Result},
    mapping::IntoActivity,
    state::Zustand,
};
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::{stream, StreamExt, TryStreamExt};
use phenomenon_type::ap::{
    ap_context,
    collection::{Collection, CollectionPage, CollectionType, PageType},
};
use sea_orm::{
    ColumnTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Related,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const ACTIVITIES_PER_PAGE: u64 = 10;

#[derive(Deserialize, Serialize)]
pub struct OutboxQuery {
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default)]
    page: bool,
}

pub async fn get(
    State(state): State<Zustand>,
    OriginalUri(original_uri): OriginalUri,
    Path(username): Path<String>,
    Query(query): Query<OutboxQuery>,
) -> Result<Response> {
    let Some(account) = <user::Entity as Related<account::Entity>>::find_related()
        .filter(user::Column::Username.eq(username.as_str()))
        .one(&state.db_conn)
        .await?
    else {
        return Err(Error::UserNotFound);
    };

    let base_url = format!("https://{}/users/{}/outbox", state.config.domain, username);
    let base_query = account
        .find_related(post::Entity)
        .filter(post::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]));

    if query.page {
        let mut page_query = base_query;
        if let Some(max_id) = query.max_id {
            page_query = page_query.filter(post::Column::Id.lt(max_id));
        }
        if let Some(min_id) = query.min_id {
            page_query = page_query.filter(post::Column::Id.gt(min_id));
        }

        let posts = page_query
            .limit(ACTIVITIES_PER_PAGE)
            .order_by_desc(post::Column::Id)
            .all(&state.db_conn)
            .await?;

        let id = format!("{}{}", state.config.domain, original_uri);
        let prev = format!(
            "{base_url}?page=true&min_id={}",
            posts.first().map_or(Uuid::max(), |post| post.id)
        );
        let next = format!(
            "{base_url}?page=true&max_id={}",
            posts.last().map_or(Uuid::nil(), |post| post.id)
        );
        let ordered_items = stream::iter(posts)
            .then(|post| post.into_activity(&state))
            .and_then(
                |activity| async move { serde_json::to_value(&activity).map_err(Error::from) },
            )
            .try_collect()
            .await?;

        Ok(Json(CollectionPage {
            context: ap_context(),
            r#type: PageType::OrderedCollectionPage,
            id,
            prev,
            next,
            part_of: base_url,
            ordered_items,
        })
        .into_response())
    } else {
        let public_post_count = base_query.count(&state.db_conn).await?;
        let first = format!("{base_url}?page=true");
        let last = format!("{base_url}?page=true&min_id={}", Uuid::nil());

        Ok(Json(Collection {
            context: ap_context(),
            id: base_url,
            r#type: CollectionType::OrderedCollection,
            total_items: public_post_count,
            first: Some(first),
            last: Some(last),
        })
        .into_response())
    }
}
