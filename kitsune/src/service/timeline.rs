use crate::error::{Error, Result};
use futures_util::{Stream, TryStreamExt};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, TypedBuilder)]
pub struct GetHome {
    fetching_account_id: Uuid,

    #[builder(default)]
    max_id: Option<Uuid>,

    #[builder(default)]
    min_id: Option<Uuid>,
}

#[derive(Clone, TypedBuilder)]
pub struct GetPublic {
    #[builder(default)]
    max_id: Option<Uuid>,

    #[builder(default)]
    min_id: Option<Uuid>,

    #[builder(default)]
    only_local: bool,

    #[builder(default)]
    only_remote: bool,
}

#[derive(Clone, TypedBuilder)]
pub struct TimelineService {
    db_conn: DatabaseConnection,
}

impl TimelineService {
    /// Get a stream of posts that resemble the users home timeline
    pub async fn get_home(
        &self,
        get_home: GetHome,
    ) -> Result<impl Stream<Item = Result<posts::Model>> + '_> {
        let mut query = Posts::find()
            .filter(
                // Post is owned by the user
                posts::Column::AccountId
                    .eq(get_home.fetching_account_id)
                    // User is following the author and the post is not a direct message
                    .or(posts::Column::Visibility
                        .is_in([
                            Visibility::Public,
                            Visibility::Unlisted,
                            Visibility::FollowerOnly,
                        ])
                        .and(
                            posts::Column::AccountId.in_subquery(
                                AccountsFollowers::find()
                                    .filter(
                                        accounts_followers::Column::FollowerId
                                            .eq(get_home.fetching_account_id),
                                    )
                                    .filter(accounts_followers::Column::ApprovedAt.is_not_null())
                                    .select_only()
                                    .column(accounts_followers::Column::AccountId)
                                    .into_query(),
                            ),
                        ))
                    // User is mentioned in the post
                    .or(posts::Column::Id.in_subquery(
                        PostsMentions::find()
                            .filter(
                                posts_mentions::Column::AccountId.eq(get_home.fetching_account_id),
                            )
                            .select_only()
                            .column(posts_mentions::Column::PostId)
                            .into_query(),
                    )),
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
