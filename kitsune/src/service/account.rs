use super::url::UrlService;
use crate::error::{ApiError, Error, Result};
use chrono::Utc;
use derive_builder::Builder;
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
use uuid::Uuid;

#[derive(Builder, Clone)]
pub struct Follow {
    account_id: Uuid,
    follower_id: Uuid,
}

impl Follow {
    #[must_use]
    pub fn builder() -> FollowBuilder {
        FollowBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct GetPosts {
    /// ID of the account whose posts are getting fetched
    account_id: Uuid,

    /// ID of the account that is requesting the posts
    #[builder(default, setter(strip_option))]
    fetching_account_id: Option<Uuid>,

    /// Smallest ID
    ///
    /// Used for pagination
    #[builder(default, setter(strip_option))]
    min_id: Option<Uuid>,

    /// Largest ID
    ///
    /// Used for pagination
    #[builder(default, setter(strip_option))]
    max_id: Option<Uuid>,
}

impl GetPosts {
    #[must_use]
    pub fn builder() -> GetPostsBuilder {
        GetPostsBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct Unfollow {
    /// Account that is being followed
    account_id: Uuid,

    /// Account that is following
    follower_id: Uuid,
}

impl Unfollow {
    #[must_use]
    pub fn builder() -> UnfollowBuilder {
        UnfollowBuilder::default()
    }
}

#[derive(Builder, Clone)]
pub struct AccountService {
    db_conn: DatabaseConnection,
    url_service: UrlService,
}

impl AccountService {
    #[must_use]
    pub fn builder() -> AccountServiceBuilder {
        AccountServiceBuilder::default()
    }

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

        AccountsFollowers::insert(follow_model.into_active_model())
            .exec_without_returning(&self.db_conn)
            .await?;

        // TODO: Federate this follow

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

        let exec_result = AccountsFollowers::delete_many()
            .filter(accounts_followers::Column::AccountId.eq(account.id))
            .filter(accounts_followers::Column::FollowerId.eq(follower.id))
            .exec(&self.db_conn)
            .await?;

        if exec_result.rows_affected != 0 {
            // TODO: Federate this unfollow
        }

        Ok((account, follower))
    }
}
