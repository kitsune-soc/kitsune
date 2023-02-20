use crate::error::{Error, Result};
use derive_builder::Builder;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    entity::{posts, prelude::Posts},
    r#trait::{PermissionCheck, PostPermissionCheckExt},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

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
pub struct AccountService {
    db_conn: DatabaseConnection,
}

impl AccountService {
    #[must_use]
    pub fn builder() -> AccountServiceBuilder {
        AccountServiceBuilder::default()
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
}
