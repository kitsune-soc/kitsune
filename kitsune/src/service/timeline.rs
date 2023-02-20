use crate::error::{Error, Result};
use derive_builder::Builder;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    custom::Visibility,
    entity::{
        accounts, accounts_followers, posts, posts_mentions,
        prelude::{AccountsFollowers, Posts},
    },
    r#trait::{PermissionCheck, PostPermissionCheckExt},
};
use sea_orm::{
    sea_query::{Expr, IntoCondition},
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, RelationTrait,
};
use uuid::Uuid;

#[derive(Builder, Clone)]
pub struct GetHome {
    fetching_account_id: Uuid,

    #[builder(default, setter(strip_option))]
    max_id: Option<Uuid>,

    #[builder(default, setter(strip_option))]
    min_id: Option<Uuid>,
}

impl GetHome {
    #[must_use]
    pub fn builder() -> GetHomeBuilder {
        GetHomeBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct GetPublic {
    #[builder(default, setter(strip_option))]
    max_id: Option<Uuid>,

    #[builder(default, setter(strip_option))]
    min_id: Option<Uuid>,

    #[builder(default)]
    only_local: bool,

    #[builder(default)]
    only_remote: bool,
}

impl GetPublic {
    #[must_use]
    pub fn builder() -> GetPublicBuilder {
        GetPublicBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct TimelineService {
    db_conn: DatabaseConnection,
}

impl TimelineService {
    #[must_use]
    pub fn builder() -> TimelineServiceBuilder {
        TimelineServiceBuilder::default()
    }

    /// Get a stream of posts that resemble the users home timeline
    pub async fn get_home(
        &self,
        get_home: GetHome,
    ) -> Result<impl Stream<Item = Result<posts::Model>> + '_> {
        let mut query = Posts::find()
            // Post is owned by the user
            .filter(posts::Column::AccountId.eq(get_home.fetching_account_id))
            // User is following the author and the post is not a direct message
            .join(
                JoinType::LeftJoin,
                posts::Relation::Accounts
                    .def()
                    .on_condition(move |posts_left, accounts_right| {
                        Expr::col((posts_left, posts::Column::Visibility))
                            .is_in([
                                Visibility::Public,
                                Visibility::Unlisted,
                                Visibility::FollowerOnly,
                            ])
                            .and(
                                Expr::col((accounts_right, accounts::Column::Id)).in_subquery(
                                    AccountsFollowers::find()
                                        .filter(
                                            accounts_followers::Column::FollowerId
                                                .eq(get_home.fetching_account_id),
                                        )
                                        .filter(
                                            accounts_followers::Column::ApprovedAt.is_not_null(),
                                        )
                                        .select_only()
                                        .column(accounts_followers::Column::AccountId)
                                        .into_query(),
                                ),
                            )
                            .into_condition()
                    }),
            )
            // User is mentioned in the post
            .join(
                JoinType::LeftJoin,
                posts_mentions::Relation::Posts.def().rev().on_condition(
                    move |_posts_left, mentions_right| {
                        Expr::col((mentions_right, posts_mentions::Column::AccountId))
                            .eq(get_home.fetching_account_id)
                            .into_condition()
                    },
                ),
            )
            .order_by_desc(posts::Column::CreatedAt);

        if let Some(max_id) = get_home.max_id {
            query = query.filter(posts::Column::Id.lt(max_id));
        }
        if let Some(min_id) = get_home.min_id {
            query = query.filter(posts::Column::Id.gt(min_id));
        }

        Ok(query.stream(&self.db_conn).await?.map_err(Error::from))
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
    ) -> Result<impl Stream<Item = Result<posts::Model>> + '_> {
        let permission_check = PermissionCheck::builder()
            .include_unlisted(false)
            .build()
            .unwrap();

        let mut query = Posts::find()
            .add_permission_checks(permission_check)
            .order_by_desc(posts::Column::CreatedAt);

        if let Some(max_id) = get_public.max_id {
            query = query.filter(posts::Column::Id.lt(max_id));
        }
        if let Some(min_id) = get_public.min_id {
            query = query.filter(posts::Column::Id.gt(min_id));
        }

        if get_public.only_local {
            query = query.filter(posts::Column::IsLocal.eq(true));
        } else if get_public.only_remote {
            query = query.filter(posts::Column::IsLocal.eq(false));
        }

        Ok(query.stream(&self.db_conn).await?.map_err(Error::from))
    }
}
