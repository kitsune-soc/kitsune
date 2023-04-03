use super::{
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    error::{ApiError, Error, Result},
    job::deliver::{follow::DeliverFollow, unfollow::DeliverUnfollow},
};
use chrono::Utc;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    entity::{
        accounts, accounts_followers, posts,
        prelude::{Accounts, AccountsFollowers, Posts},
    },
    r#trait::{PermissionCheck, PostPermissionCheckExt},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, TypedBuilder)]
pub struct Follow {
    account_id: Uuid,
    follower_id: Uuid,
}

#[derive(Clone, TypedBuilder)]
pub struct GetPosts {
    /// ID of the account whose posts are getting fetched
    account_id: Uuid,

    /// ID of the account that is requesting the posts
    #[builder(default)]
    fetching_account_id: Option<Uuid>,

    /// Smallest ID
    ///
    /// Used for pagination
    #[builder(default)]
    min_id: Option<Uuid>,

    /// Largest ID
    ///
    /// Used for pagination
    #[builder(default)]
    max_id: Option<Uuid>,
}

#[derive(Clone, TypedBuilder)]
pub struct Unfollow {
    /// Account that is being followed
    account_id: Uuid,

    /// Account that is following
    follower_id: Uuid,
}

#[derive(Clone, TypedBuilder)]
pub struct AccountService {
    db_conn: DatabaseConnection,
    job_service: JobService,
    url_service: UrlService,
}

impl AccountService {
    /// Follow an account
    ///
    /// # Returns
    ///
    /// Tuple of two account models. First model is the account the followee account, the second model is the followed account
    pub async fn follow(&self, follow: Follow) -> Result<(accounts::Model, accounts::Model)> {
        let account = Accounts::find_by_id(follow.account_id)
            .one(&self.db_conn)
            .await?
            .ok_or(ApiError::BadRequest)?;
        let follower = Accounts::find_by_id(follow.follower_id)
            .one(&self.db_conn)
            .await?
            .ok_or(ApiError::BadRequest)?;

        let id = Uuid::now_v7();
        let url = self.url_service.follow_url(id);
        let mut follow_model = accounts_followers::Model {
            id,
            account_id: account.id,
            follower_id: follower.id,
            approved_at: None,
            url,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        };

        if account.local && !account.locked {
            follow_model.approved_at = Some(Utc::now().into());
        }

        let follow_id = AccountsFollowers::insert(follow_model.into_active_model())
            .exec(&self.db_conn)
            .await?;

        if !account.local {
            self.job_service
                .enqueue(
                    Enqueue::builder()
                        .job(DeliverFollow {
                            follow_id: follow_id.last_insert_id,
                        })
                        .build(),
                )
                .await?;
        }

        Ok((account, follower))
    }

    /// Get a local account by its username
    pub async fn get_local_by_username(&self, username: &str) -> Result<Option<accounts::Model>> {
        Accounts::find()
            .filter(accounts::Column::Username.eq(username))
            .filter(accounts::Column::Local.eq(true))
            .one(&self.db_conn)
            .await
            .map_err(Error::from)
    }

    /// Get a stream of posts owned by the user
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub async fn get_posts(
        &self,
        get_posts: GetPosts,
    ) -> Result<impl Stream<Item = Result<posts::Model>> + '_> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(get_posts.fetching_account_id)
            .build()
            .unwrap();

        let mut posts_query = Posts::find()
            .filter(posts::Column::AccountId.eq(get_posts.account_id))
            .add_permission_checks(permission_check)
            .order_by_desc(posts::Column::CreatedAt);

        if let Some(min_id) = get_posts.min_id {
            posts_query = posts_query.filter(posts::Column::Id.gt(min_id));
        }

        if let Some(max_id) = get_posts.max_id {
            posts_query = posts_query.filter(posts::Column::Id.lt(max_id));
        }

        Ok(posts_query
            .stream(&self.db_conn)
            .await?
            .map_err(Error::from))
    }

    /// Undo the follow of an account
    ///
    /// # Returns
    ///
    /// Tuple of two account models. First account is the account that was being followed, second account is the account that was following
    pub async fn unfollow(&self, unfollow: Unfollow) -> Result<(accounts::Model, accounts::Model)> {
        let account = Accounts::find_by_id(unfollow.account_id)
            .one(&self.db_conn)
            .await?
            .ok_or(ApiError::BadRequest)?;
        let follower = Accounts::find_by_id(unfollow.follower_id)
            .one(&self.db_conn)
            .await?
            .ok_or(ApiError::BadRequest)?;

        let follow = AccountsFollowers::find()
            .filter(accounts_followers::Column::AccountId.eq(account.id))
            .filter(accounts_followers::Column::FollowerId.eq(follower.id))
            .one(&self.db_conn)
            .await?;

        if let Some(follow) = follow {
            if !account.local {
                self.job_service
                    .enqueue(
                        Enqueue::builder()
                            .job(DeliverUnfollow {
                                follow_id: follow.id,
                            })
                            .build(),
                    )
                    .await?;
            }
        }

        Ok((account, follower))
    }
}
