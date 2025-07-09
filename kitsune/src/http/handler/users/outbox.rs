use crate::{http::responder::ActivityPubJson, state::Zustand};
use axum::extract::{OriginalUri, Path, Query, State};
use axum_extra::either::Either;
use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, SelectableHelper};
use futures_util::{StreamExt, TryStreamExt, stream};
use kitsune_activitypub::mapping::IntoActivity;
use kitsune_db::{
    model::{account::Account, post::Post},
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::accounts,
    with_connection,
};
use kitsune_error::Result;
use kitsune_service::account::GetPosts;
use kitsune_type::ap::{
    Activity, ap_context,
    collection::{Collection, CollectionPage, CollectionType, PageType},
};
use kitsune_url::UrlService;
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
    State(state): State<Zustand>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Path(account_id): Path<Uuid>,
    Query(query): Query<OutboxQuery>,
) -> Result<Either<ActivityPubJson<CollectionPage<Activity>>, ActivityPubJson<Collection>>> {
    let account = with_connection!(state.db_pool, |db_conn| {
        use diesel_async::RunQueryDsl;

        accounts::table
            .find(account_id)
            .filter(accounts::local.eq(true))
            .select(Account::as_select())
            .get_result::<Account>(db_conn)
            .await
    })?;

    let base_url = format!("{}{}", url_service.base_url(), original_uri.path());

    if query.page {
        let get_posts = GetPosts::builder()
            .account_id(account.id)
            .max_id(query.max_id)
            .min_id(query.min_id)
            .limit(ACTIVITIES_PER_PAGE)
            .build();

        let posts: Vec<Post> = state
            .service
            .account
            .get_posts(get_posts)
            .await?
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
            .then(|post| post.into_activity(state.ap_state()))
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
        let public_post_count = with_connection!(state.db_pool, |db_conn| {
            use diesel_async::RunQueryDsl;

            Post::belonging_to(&account)
                .add_post_permission_check(PermissionCheck::default())
                .count()
                .get_result::<i64>(db_conn)
                .await
        })?;

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
