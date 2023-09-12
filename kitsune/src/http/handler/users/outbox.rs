use crate::{error::Result, http::responder::ActivityPubJson, state::AppState};
use axum::extract::{OriginalUri, Path, Query, State};
use axum_extra::either::Either;
use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{stream, StreamExt, TryStreamExt};
use kitsune_core::{
    mapping::IntoActivity,
    service::{account::GetPosts, url::UrlService},
};
use kitsune_db::{
    model::{account::Account, post::Post},
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::accounts,
};
use kitsune_type::ap::{
    ap_context,
    collection::{Collection, CollectionPage, CollectionType, PageType},
    Activity,
};
use scoped_futures::ScopedFutureExt;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

const ACTIVITIES_PER_PAGE: usize = 10;

#[derive(Deserialize, Serialize)]
pub struct OutboxQuery {
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default)]
    page: bool,
}

pub async fn get(
    State(state): State<AppState>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(account_id): Path<Uuid>,
    Query(query): Query<OutboxQuery>,
) -> Result<Either<ActivityPubJson<CollectionPage<Activity>>, ActivityPubJson<Collection>>> {
    let account = state
        .db_pool()
        .with_connection(|db_conn| {
            accounts::table
                .find(account_id)
                .filter(accounts::local.eq(true))
                .select(Account::as_select())
                .get_result::<Account>(db_conn)
                .scoped()
        })
        .await?;

    let base_url = format!("{}{}", url_service.base_url(), original_uri.path());

    if query.page {
        let get_posts = GetPosts::builder()
            .account_id(account.id)
            .max_id(query.max_id)
            .min_id(query.min_id)
            .limit(ACTIVITIES_PER_PAGE)
            .build();

        let posts: Vec<Post> = state
            .service()
            .account
            .get_posts(get_posts)
            .await?
            .try_collect()
            .await?;

        let id = format!("{}{original_uri}", url_service.base_url());
        let prev = format!(
            "{base_url}?page=true&min_id={}",
            posts.get(0).map_or(Uuid::max(), |post| post.id)
        );
        let next = format!(
            "{base_url}?page=true&max_id={}",
            posts.last().map_or(Uuid::nil(), |post| post.id)
        );
        let ordered_items = stream::iter(posts)
            .then(|post| post.into_activity(&state.core))
            .try_collect()
            .await?;

        Ok(Either::E1(ActivityPubJson(CollectionPage {
            context: ap_context(),
            r#type: PageType::OrderedCollectionPage,
            id,
            prev,
            next,
            part_of: base_url,
            ordered_items,
        })))
    } else {
        let public_post_count = state
            .db_pool()
            .with_connection(|db_conn| {
                Post::belonging_to(&account)
                    .add_post_permission_check(PermissionCheck::default())
                    .count()
                    .get_result::<i64>(db_conn)
                    .scoped()
            })
            .await?;

        let first = format!("{base_url}?page=true");
        let last = format!("{base_url}?page=true&min_id={}", Uuid::nil());

        Ok(Either::E2(ActivityPubJson(Collection {
            context: ap_context(),
            id: base_url,
            r#type: CollectionType::OrderedCollection,
            total_items: public_post_count as u64,
            first: Some(first),
            last: Some(last),
        })))
    }
}
