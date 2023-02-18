use crate::{
    error::{ApiError, Error, Result},
    mapping::IntoActivity,
    service::account::GetPosts,
    state::Zustand,
};
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::{stream, StreamExt, TryStreamExt};
use kitsune_db::{
    entity::{
        posts,
        prelude::{Accounts, Posts, Users},
        users,
    },
    r#trait::PostPermissionCheckExt,
};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionPage, CollectionType, PageType},
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Related};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const ACTIVITIES_PER_PAGE: usize = 10;

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
    let Some(account) = <Users as Related<Accounts>>::find_related()
        .filter(users::Column::Username.eq(username.as_str()))
        .one(&state.db_conn)
        .await?
    else {
        return Err(ApiError::NotFound.into());
    };

    let base_url = format!("https://{}{}", state.config.domain, original_uri.path());
    let base_query = Posts::find()
        .filter(posts::Column::AccountId.eq(account.id))
        .add_permission_checks(None);

    if query.page {
        let mut get_posts = GetPosts::builder().account_id(account.id).clone();
        if let Some(max_id) = query.max_id {
            get_posts.max_id(max_id);
        }
        if let Some(min_id) = query.min_id {
            get_posts.min_id(min_id);
        }
        let get_posts = get_posts.build().unwrap();

        let posts: Vec<posts::Model> = state
            .service
            .account
            .get_posts(get_posts)
            .await?
            .take(ACTIVITIES_PER_PAGE)
            .try_collect()
            .await?;

        let id = format!("{}{original_uri}", state.config.domain);
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
