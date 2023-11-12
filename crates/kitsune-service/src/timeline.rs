use super::LimitContext;
use crate::error::{Error, Result};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use garde::Validate;
use kitsune_db::{
    model::post::{Post, Visibility},
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{accounts_follows, posts, posts_mentions},
    PgPool,
};
use scoped_futures::ScopedFutureExt;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder, Validate)]
#[garde(context(LimitContext as ctx))]
pub struct GetHome {
    #[garde(skip)]
    fetching_account_id: Uuid,

    #[garde(range(max = ctx.limit))]
    limit: usize,

    #[builder(default)]
    #[garde(skip)]
    max_id: Option<Uuid>,

    #[builder(default)]
    #[garde(skip)]
    since_id: Option<Uuid>,

    #[builder(default)]
    #[garde(skip)]
    min_id: Option<Uuid>,
}

#[derive(Clone, TypedBuilder, Validate)]
#[garde(context(LimitContext as ctx))]
pub struct GetPublic {
    #[garde(range(max = ctx.limit))]
    limit: usize,

    #[builder(default)]
    #[garde(skip)]
    max_id: Option<Uuid>,

    #[builder(default)]
    #[garde(skip)]
    since_id: Option<Uuid>,

    #[builder(default)]
    #[garde(skip)]
    min_id: Option<Uuid>,

    #[builder(default)]
    #[garde(skip)]
    only_local: bool,

    #[builder(default)]
    #[garde(skip)]
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
        get_home.validate(&LimitContext::default())?;

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
            .limit(get_home.limit as i64)
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
            .with_connection(|db_conn| {
                async move {
                    Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from))
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
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
        get_public.validate(&LimitContext::default())?;

        let permission_check = PermissionCheck::builder().include_unlisted(false).build();

        let mut query = posts::table
            .add_post_permission_check(permission_check)
            .order(posts::id.desc())
            .limit(get_public.limit as i64)
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
            .with_connection(|db_conn| {
                async move {
                    Ok::<_, Error>(query.load_stream(db_conn).await?.map_err(Error::from))
                }
                .scoped()
            })
            .await
            .map_err(Error::from)
    }
}
