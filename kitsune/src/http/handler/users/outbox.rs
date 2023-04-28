use crate::{
    error::{ApiError, Error, Result},
    http::responder::ActivityPubJson,
    mapping::IntoActivity,
    service::{account::GetPosts, url::UrlService},
    state::Zustand,
};
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::{IntoResponse, Response},
};
use futures_util::{stream, StreamExt, TryStreamExt};
use kitsune_db::{
    entity::{
        accounts, posts,
        prelude::{Accounts, Posts},
    },
    r#trait::{PermissionCheck, PostPermissionCheckExt},
};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionPage, CollectionType, PageType},
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
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
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(account_id): Path<Uuid>,
    Query(query): Query<OutboxQuery>,
) -> Result<Response> {
    let Some(account) = Accounts::find_by_id(account_id)
        .filter(accounts::Column::Local.eq(true))
        .one(&state.db_conn)
        .await?
    else {
        return Err(ApiError::NotFound.into());
    };

    let base_url = format!("{}{}", url_service.base_url(), original_uri.path());

    if query.page {
        let get_posts = GetPosts::builder()
            .account_id(account.id)
            .max_id(query.max_id)
            .min_id(query.min_id)
            .build();

        let posts: Vec<posts::Model> = state
            .service
            .account
            .get_posts(get_posts)
            .await?
            .take(ACTIVITIES_PER_PAGE)
            .try_collect()
            .await?;

        let id = format!("{}{original_uri}", url_service.base_url());
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

        Ok(ActivityPubJson(CollectionPage {
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
        let public_post_count = Posts::find()
            .filter(posts::Column::AccountId.eq(account.id))
            .add_permission_checks(PermissionCheck::default())
            .count(&state.db_conn)
            .await?;

        let first = format!("{base_url}?page=true");
        let last = format!("{base_url}?page=true&min_id={}", Uuid::nil());

        Ok(ActivityPubJson(Collection {
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
