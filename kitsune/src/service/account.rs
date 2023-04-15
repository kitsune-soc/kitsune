use super::{
    attachment::{AttachmentService, Upload},
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    activitypub::Fetcher,
    error::{ApiError, Error, Result},
    job::deliver::{
        follow::DeliverFollow,
        unfollow::DeliverUnfollow,
        update::{DeliverUpdate, UpdateEntity},
    },
    webfinger::Webfinger,
};
use bytes::Bytes;
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
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Clone, TypedBuilder)]
pub struct Follow {
    account_id: Uuid,
    follower_id: Uuid,
}

#[derive(Clone, TypedBuilder)]
pub struct GetUser<'a> {
    username: &'a str,
    #[builder(default)]
    domain: Option<&'a str>,
    #[builder(default = true)]
    use_webfinger: bool,
}

impl<'a> From<&'a str> for GetUser<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().username(value).build()
    }
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

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Update<A, H> {
    account_id: Uuid,
    #[builder(default, setter(strip_option))]
    display_name: Option<String>,
    #[builder(default, setter(strip_option))]
    note: Option<String>,
    #[builder(default, setter(strip_option))]
    avatar: Option<Upload<A>>,
    #[builder(default, setter(strip_option))]
    header: Option<Upload<H>>,
    #[builder(default, setter(strip_option))]
    locked: Option<bool>,
}

impl<A, H> Update<A, H> {
    #[must_use]
    pub fn builder() -> UpdateBuilder<A, H> {
        UpdateBuilder::default()
    }
}

#[derive(Clone, TypedBuilder)]
pub struct AccountService {
    attachment_service: AttachmentService,
    db_conn: DatabaseConnection,
    fetcher: Fetcher,
    job_service: JobService,
    url_service: UrlService,
    webfinger: Webfinger,
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

    /// Get an account by its username and domain
    pub async fn get(&self, get_user: GetUser<'_>) -> Result<Option<accounts::Model>> {
        if let Some(domain) = get_user.domain {
            if let Some(account) = Accounts::find()
                .filter(accounts::Column::Username.eq(get_user.username))
                .filter(accounts::Column::Domain.eq(domain))
                .one(&self.db_conn)
                .await?
            {
                return Ok(Some(account));
            } else if !get_user.use_webfinger {
                return Ok(None);
            }

            let Some(actor_url) = self.webfinger.fetch_actor_url(get_user.username, domain).await? else {
                return Ok(None)
            };

            self.fetcher
                .fetch_actor(actor_url.as_str().into())
                .await
                .map(Some)
                .map_err(Error::from)
        } else {
            Accounts::find()
                .filter(
                    accounts::Column::Username
                        .eq(get_user.username)
                        .and(accounts::Column::Local.eq(true)),
                )
                .one(&self.db_conn)
                .await
                .map_err(Error::from)
        }
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

    pub async fn update<A, H>(&self, update: Update<A, H>) -> Result<accounts::Model>
    where
        A: Stream<Item = kitsune_storage::Result<Bytes>> + Send + 'static,
        H: Stream<Item = kitsune_storage::Result<Bytes>> + Send + 'static,
    {
        let mut active_model = accounts::ActiveModel {
            id: ActiveValue::Set(update.account_id),
            ..Default::default()
        };

        if let Some(display_name) = update.display_name {
            active_model.display_name = ActiveValue::Set(Some(display_name));
        }
        if let Some(note) = update.note {
            active_model.note = ActiveValue::Set(Some(note));
        }
        if let Some(avatar) = update.avatar {
            let media_attachment = self.attachment_service.upload(avatar).await?;
            active_model.avatar_id = ActiveValue::Set(Some(media_attachment.id));
        }
        if let Some(header) = update.header {
            let media_attachment = self.attachment_service.upload(header).await?;
            active_model.header_id = ActiveValue::Set(Some(media_attachment.id));
        }
        if let Some(locked) = update.locked {
            active_model.locked = ActiveValue::Set(locked);
        }

        let updated_account = Accounts::update(active_model).exec(&self.db_conn).await?;

        self.job_service
            .enqueue(
                Enqueue::builder()
                    .job(DeliverUpdate {
                        entity: UpdateEntity::Account,
                        id: updated_account.id,
                    })
                    .build(),
            )
            .await?;

        Ok(updated_account)
    }
}
