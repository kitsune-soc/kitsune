use super::{
    attachment::{AttachmentService, Upload},
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    activitypub::Fetcher,
    error::{Error, Result},
    job::deliver::{
        follow::DeliverFollow,
        unfollow::DeliverUnfollow,
        update::{DeliverUpdate, UpdateEntity},
    },
    webfinger::Webfinger,
};
use bytes::Bytes;
use derive_builder::Builder;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use kitsune_db::{
    model::{account::Account, follower::NewFollow},
    schema::{accounts, accounts_follows},
    PgPool,
};
use time::OffsetDateTime;
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
    db_conn: PgPool,
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
    pub async fn follow(&self, follow: Follow) -> Result<(Account, Account)> {
        let mut db_conn = self.db_conn.get().await?;

        let (account, follower) = tokio::try_join!(
            accounts::table
                .find(follow.account_id)
                .get_result(&mut db_conn),
            accounts::table
                .find(follow.follower_id)
                .get_result(&mut db_conn)
        );

        let id = Uuid::now_v7();
        let url = self.url_service.follow_url(id);
        let mut follow_model = NewFollow {
            id,
            account_id: account.id,
            follower_id: follower.id,
            approved_at: None,
            url: url.as_str(),
            created_at: None,
        };

        if account.local && !account.locked {
            follow_model.approved_at = Some(OffsetDateTime::now_utc());
        }

        let follow_id = diesel::insert_into(accounts_follows::table)
            .values(follow_model)
            .select(accounts_follows::id)
            .execute(&mut db_conn)
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
    pub async fn get(&self, get_user: GetUser<'_>) -> Result<Option<Account>> {
        if let Some(domain) = get_user.domain {
            if let Some(account) = accounts::table
                .filter(
                    accounts::username
                        .eq(get_user.username)
                        .and(accounts::domain.eq(domain)),
                )
                .optional()
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
            accounts::table
                .filter(
                    accounts::username
                        .eq(get_user.username)
                        .and(accounts::local.eq(true)),
                )
                .first(&self.db_cnn)
                .await
                .map_err(Error::from)
        }
    }

    /// Get an account by its ID
    pub async fn get_by_id(&self, account_id: Uuid) -> Result<Option<Account>> {
        accounts::table
            .find(account_id)
            .get_result(&mut self.db_conn.get().await?)
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
    pub async fn unfollow(&self, unfollow: Unfollow) -> Result<(Account, Account)> {
        let mut db_conn = self.db_conn.get().await?;

        let (account, follower) = tokio::try_join!(
            accounts::table
                .find(unfollow.account_id)
                .get_result(&mut db_conn),
            accounts::table
                .find(unfollow.follower_id)
                .get_result(&mut db_conn)
        );

        let follow = accounts_follows::table
            .filter(
                accounts_follows::account_id
                    .eq(account.id)
                    .and(accounts_follows::follower_id.eq(follower.id)),
            )
            .optional()
            .get_result(&self.db_conn)
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

    pub async fn update<A, H>(&self, update: Update<A, H>) -> Result<Account>
    where
        A: Stream<Item = kitsune_storage::Result<Bytes>> + Send + 'static,
        H: Stream<Item = kitsune_storage::Result<Bytes>> + Send + 'static,
    {
        let mut update_query = diesel::update(accounts::table.find(update.account_id));

        if let Some(display_name) = update.display_name {
            update_query = update_query.set(accounts::display_name.eq(display_name));
        }
        if let Some(note) = update.note {
            update_query = update_query.set(accounts::note.eq(note));
        }
        if let Some(avatar) = update.avatar {
            let media_attachment = self.attachment_service.upload(avatar).await?;
            update_query = update_query.set(accounts::avatar_id.eq(media_attachment.id));
        }
        if let Some(header) = update.header {
            let media_attachment = self.attachment_service.upload(header).await?;
            update_query = update_query.set(accounts::header_id.eq(media_attachment.id));
        }
        if let Some(locked) = update.locked {
            update_query = update_query.set(accounts::locked.eq(locked));
        }

        let updated_account = update_query.execute(&mut self.db_conn.get().await?).await?;

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
