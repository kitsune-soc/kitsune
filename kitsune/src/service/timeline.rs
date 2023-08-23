use crate::{
    consts::{API_DEFAULT_LIMIT, API_MAX_LIMIT},
    error::{Error, Result},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    model::post::{Post, Visibility},
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{accounts_follows, posts, posts_mentions},
    PgPool,
};
use speedy_uuid::Uuid;
use std::cmp::min;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct GetHome {
    fetching_account_id: Uuid,

    #[builder(default = API_DEFAULT_LIMIT)]
    limit: usize,

    #[builder(default)]
    max_id: Option<Uuid>,

    #[builder(default)]
    since_id: Option<Uuid>,

    #[builder(default)]
    min_id: Option<Uuid>,
}

#[derive(Clone, TypedBuilder)]
pub struct GetPublic {
    #[builder(default = API_DEFAULT_LIMIT)]
    limit: usize,

    #[builder(default)]
    max_id: Option<Uuid>,

    #[builder(default)]
    since_id: Option<Uuid>,

    #[builder(default)]
    min_id: Option<Uuid>,

    #[builder(default)]
    only_local: bool,

    #[builder(default)]
    only_remote: bool,
}

#[derive(Clone, TypedBuilder)]
pub struct TimelineService {
    db_pool: PgPool,
}

impl TimelineService {
    /// Get a stream of posts that resemble the users home timeline
    pub async fn get_home(
        &self,
        get_home: GetHome,
    ) -> Result<impl Stream<Item = Result<Post>> + '_> {
        let mut query = posts::table
            .filter(
                // Post is owned by the user
                posts::account_id
                    .eq(get_home.fetching_account_id)
                    // User is following the author and the post is not a direct message
                    .or(posts::visibility
                        .eq_any([
                            Visibility::Public,
                            Visibility::Unlisted,
                            Visibility::FollowerOnly,
                        ])
                        .and(
                            posts::account_id.eq_any(
                                accounts_follows::table
                                    .filter(
                                        accounts_follows::follower_id
                                            .eq(get_home.fetching_account_id),
                                    )
                                    .filter(accounts_follows::approved_at.is_not_null())
                                    .select(accounts_follows::account_id),
                            ),
                        ))
                    // User is mentioned in the post
                    .or(posts::id.eq_any(
                        posts_mentions::table
                            .filter(posts_mentions::account_id.eq(get_home.fetching_account_id))
                            .select(posts_mentions::post_id),
                    )),
            )
            .order(posts::id.desc())
            .limit(min(get_home.limit, API_MAX_LIMIT) as i64)
            .select(Post::as_select())
            .into_boxed();

        if let Some(max_id) = get_home.max_id {
            query = query.filter(posts::id.lt(max_id));
        }
        if let Some(since_id) = get_home.since_id {
            query = query.filter(posts::id.gt(since_id));
        }
        if let Some(min_id) = get_home.min_id {
            query = query.filter(posts::id.gt(min_id)).order(posts::id.asc());
        }

        self.db_pool
            .with_connection(|mut db_conn| async move {
                Ok(query.load_stream(&mut db_conn).await?.map_err(Error::from))
            })
            .await
    }

    /// Get a stream of public posts
    ///
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub async fn get_public(
        &self,
        get_public: GetPublic,
    ) -> Result<impl Stream<Item = Result<Post>> + '_> {
        let permission_check = PermissionCheck::builder()
            .include_unlisted(false)
            .build()
            .unwrap();

        let mut query = posts::table
            .add_post_permission_check(permission_check)
            .order(posts::id.desc())
            .limit(min(get_public.limit, API_MAX_LIMIT) as i64)
            .select(Post::as_select())
            .into_boxed();

        if let Some(max_id) = get_public.max_id {
            query = query.filter(posts::id.lt(max_id));
        }
        if let Some(since_id) = get_public.since_id {
            query = query.filter(posts::id.gt(since_id));
        }
        if let Some(min_id) = get_public.min_id {
            query = query.filter(posts::id.gt(min_id)).order(posts::id.asc());
        }

        if get_public.only_local {
            query = query.filter(posts::is_local.eq(true));
        } else if get_public.only_remote {
            query = query.filter(posts::is_local.eq(false));
        }

        self.db_pool
            .with_connection(|mut db_conn| async move {
                Ok(query.load_stream(&mut db_conn).await?.map_err(Error::from))
            })
            .await
    }
}
