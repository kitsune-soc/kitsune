use super::{
    attachment::{AttachmentService, Upload},
    job::{Enqueue, JobService},
    url::UrlService,
};
use crate::{
    activitypub::Fetcher,
    consts::{API_DEFAULT_LIMIT, API_MAX_LIMIT},
    error::{Error, Result},
    job::deliver::{
        accept::DeliverAccept,
        follow::DeliverFollow,
        reject::DeliverReject,
        unfollow::DeliverUnfollow,
        update::{DeliverUpdate, UpdateEntity},
    },
    sanitize::CleanHtmlExt,
    try_join,
    webfinger::Webfinger,
};
use bytes::Bytes;
use derive_builder::Builder;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
    SelectableHelper,
};
use diesel_async::RunQueryDsl;
use futures_util::{Stream, TryStreamExt};
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    model::{
        account::{Account, UpdateAccount},
        follower::Follow as DbFollow,
        follower::NewFollow,
        post::Post,
    },
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{accounts, accounts_follows, posts},
    PgPool,
};
use speedy_uuid::Uuid;
use std::cmp::min;
use typed_builder::TypedBuilder;

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

    /// Limit of returned posts
    #[builder(default = API_DEFAULT_LIMIT)]
    limit: usize,

    /// Smallest ID, return results starting from this ID
    ///
    /// Used for pagination
    #[builder(default)]
    min_id: Option<Uuid>,

    /// Smallest ID, return highest results
    ///
    /// Used for pagination
    #[builder(default)]
    since_id: Option<Uuid>,

    /// Largest ID
    ///
    /// Used for pagination
    #[builder(default)]
    max_id: Option<Uuid>,
}

#[derive(Clone, TypedBuilder)]
pub struct GetFollowRequests {
    /// ID of the account whose follow requests are getting fetched
    account_id: Uuid,

    /// Limit of returned posts
    #[builder(default = API_DEFAULT_LIMIT)]
    limit: usize,

    /// Smallest ID
    ///
    /// Used for pagination
    #[builder(default)]
    since_id: Option<Uuid>,

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
pub struct FollowRequest {
    /// Account that is the target of the follow request
    account_id: Uuid,

    /// Account that is sending the follow request
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
    db_pool: PgPool,
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
        let (account, follower) = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                let account_fut = accounts::table
                    .find(follow.account_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                let follower_fut = accounts::table
                    .find(follow.follower_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                try_join!(account_fut, follower_fut)
            })
            .await?;

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
            follow_model.approved_at = Some(Timestamp::now_utc());
        }

        let follow_id = self
            .db_pool
            .with_connection(|mut db_conn| {
                diesel::insert_into(accounts_follows::table)
                    .values(follow_model)
                    .returning(accounts_follows::id)
                    .get_result(&mut db_conn)
            })
            .await?;

        if !account.local {
            self.job_service
                .enqueue(Enqueue::builder().job(DeliverFollow { follow_id }).build())
                .await?;
        }

        Ok((account, follower))
    }

    /// Get an account by its username and domain
    pub async fn get(&self, get_user: GetUser<'_>) -> Result<Option<Account>> {
        if let Some(domain) = get_user.domain {
            let account = self
                .db_pool
                .with_connection(|mut db_conn| async move {
                    accounts::table
                        .filter(
                            accounts::username
                                .eq(get_user.username)
                                .and(accounts::domain.eq(domain)),
                        )
                        .select(Account::as_select())
                        .get_result(&mut db_conn)
                        .await
                        .optional()
                })
                .await?;

            if let Some(account) = account {
                return Ok(Some(account));
            } else if !get_user.use_webfinger {
                return Ok(None);
            }

            let Some(actor_url) = self
                .webfinger
                .fetch_actor_url(get_user.username, domain)
                .await?
            else {
                return Ok(None);
            };

            self.fetcher
                .fetch_actor(actor_url.as_str().into())
                .await
                .map(Some)
                .map_err(Error::from)
        } else {
            self.db_pool
                .with_connection(|mut db_conn| async move {
                    accounts::table
                        .filter(
                            accounts::username
                                .eq(get_user.username)
                                .and(accounts::local.eq(true)),
                        )
                        .select(Account::as_select())
                        .first(&mut db_conn)
                        .await
                        .optional()
                })
                .await
                .map_err(Error::from)
        }
    }

    /// Get an account by its ID
    pub async fn get_by_id(&self, account_id: Uuid) -> Result<Option<Account>> {
        self.db_pool
            .with_connection(|mut db_conn| async move {
                accounts::table
                    .find(account_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn)
                    .await
                    .optional()
            })
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
    ) -> Result<impl Stream<Item = Result<Post>> + '_> {
        let permission_check = PermissionCheck::builder()
            .fetching_account_id(get_posts.fetching_account_id)
            .build()
            .unwrap();

        let mut query = posts::table
            .filter(posts::account_id.eq(get_posts.account_id))
            .add_post_permission_check(permission_check)
            .select(Post::as_select())
            .order(posts::id.desc())
            .limit(min(get_posts.limit, API_MAX_LIMIT) as i64)
            .into_boxed();

        if let Some(max_id) = get_posts.max_id {
            query = query.filter(posts::id.lt(max_id));
        }
        if let Some(since_id) = get_posts.since_id {
            query = query.filter(posts::id.gt(since_id));
        }
        if let Some(min_id) = get_posts.min_id {
            query = query.filter(posts::id.gt(min_id)).order(posts::id.asc());
        }

        self.db_pool
            .with_connection(|mut db_conn| async move {
                Ok::<_, Error>(query.load_stream(&mut db_conn).await?.map_err(Error::from))
            })
            .await
            .map_err(Error::from)
    }

    /// Undo the follow of an account
    ///
    /// # Returns
    ///
    /// Tuple of two account models. First account is the account that was being followed, second account is the account that was following
    pub async fn unfollow(&self, unfollow: Unfollow) -> Result<(Account, Account)> {
        let (account, follower) = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                let account_fut = accounts::table
                    .find(unfollow.account_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                let follower_fut = accounts::table
                    .find(unfollow.follower_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                try_join!(account_fut, follower_fut)
            })
            .await?;

        let follow = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                accounts_follows::table
                    .filter(
                        accounts_follows::account_id
                            .eq(account.id)
                            .and(accounts_follows::follower_id.eq(follower.id)),
                    )
                    .get_result::<DbFollow>(&mut db_conn)
                    .await
                    .optional()
            })
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

    pub async fn get_follow_requests(
        &self,
        get_follow_requests: GetFollowRequests,
    ) -> Result<impl Stream<Item = Result<Account>> + '_> {
        let mut query = accounts_follows::table
            .inner_join(accounts::table.on(accounts_follows::follower_id.eq(accounts::id)))
            .filter(
                accounts_follows::account_id
                    .eq(get_follow_requests.account_id)
                    .and(accounts_follows::approved_at.is_null()),
            )
            .select(Account::as_select())
            .order(accounts::id.desc())
            .limit(min(get_follow_requests.limit, API_MAX_LIMIT) as i64)
            .into_boxed();

        if let Some(since_id) = get_follow_requests.since_id {
            query = query.filter(accounts::id.gt(since_id));
        }

        if let Some(max_id) = get_follow_requests.max_id {
            query = query.filter(accounts::id.lt(max_id));
        }

        self.db_pool
            .with_connection(|mut db_conn| async move {
                Ok::<_, Error>(query.load_stream(&mut db_conn).await?.map_err(Error::from))
            })
            .await
            .map_err(Error::from)
    }

    pub async fn accept_follow_request(
        &self,
        follow_request: FollowRequest,
    ) -> Result<Option<(Account, Account)>> {
        let (account, follower) = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                let account_fut = accounts::table
                    .find(follow_request.account_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                let follower_fut = accounts::table
                    .find(follow_request.follower_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                try_join!(account_fut, follower_fut)
            })
            .await?;

        let follow = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                accounts_follows::table
                    .filter(
                        accounts_follows::account_id
                            .eq(account.id)
                            .and(accounts_follows::follower_id.eq(follower.id)),
                    )
                    .get_result::<DbFollow>(&mut db_conn)
                    .await
                    .optional()
            })
            .await?;

        if let Some(follow) = follow {
            let now = Timestamp::now_utc();

            self.db_pool
                .with_connection(|mut db_conn| {
                    diesel::update(&follow)
                        .set((
                            accounts_follows::approved_at.eq(now),
                            accounts_follows::updated_at.eq(now),
                        ))
                        .execute(&mut db_conn)
                })
                .await?;

            if !account.local {
                self.job_service
                    .enqueue(
                        Enqueue::builder()
                            .job(DeliverAccept {
                                follow_id: follow.id,
                            })
                            .build(),
                    )
                    .await?;
            }
        } else {
            return Ok(None);
        }

        Ok(Some((account, follower)))
    }

    pub async fn reject_follow_request(
        &self,
        follow_request: FollowRequest,
    ) -> Result<Option<(Account, Account)>> {
        let (account, follower) = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                let account_fut = accounts::table
                    .find(follow_request.account_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                let follower_fut = accounts::table
                    .find(follow_request.follower_id)
                    .select(Account::as_select())
                    .get_result(&mut db_conn);

                try_join!(account_fut, follower_fut)
            })
            .await?;

        let follow = self
            .db_pool
            .with_connection(|mut db_conn| async move {
                accounts_follows::table
                    .filter(
                        accounts_follows::account_id
                            .eq(account.id)
                            .and(accounts_follows::follower_id.eq(follower.id)),
                    )
                    .get_result::<DbFollow>(&mut db_conn)
                    .await
                    .optional()
            })
            .await?;

        if let Some(follow) = follow {
            if account.local {
                self.db_pool
                    .with_connection(|mut db_conn| diesel::delete(&follow).execute(&mut db_conn))
                    .await?;
            } else {
                self.job_service
                    .enqueue(
                        Enqueue::builder()
                            .job(DeliverReject {
                                follow_id: follow.id,
                            })
                            .build(),
                    )
                    .await?;
            }
        } else {
            return Ok(None);
        }

        Ok(Some((account, follower)))
    }

    pub async fn update<A, H>(&self, mut update: Update<A, H>) -> Result<Account>
    where
        A: Stream<Item = kitsune_storage::Result<Bytes>> + Send + 'static,
        H: Stream<Item = kitsune_storage::Result<Bytes>> + Send + 'static,
    {
        let mut changeset = UpdateAccount::default();

        if let Some(ref mut display_name) = update.display_name {
            display_name.clean_html();
            changeset = UpdateAccount {
                display_name: Some(display_name),
                ..changeset
            };
        }
        if let Some(ref mut note) = update.note {
            note.clean_html();
            changeset = UpdateAccount {
                note: Some(note),
                ..changeset
            };
        }
        if let Some(avatar) = update.avatar {
            let media_attachment = self.attachment_service.upload(avatar).await?;
            changeset = UpdateAccount {
                avatar_id: Some(media_attachment.id),
                ..changeset
            };
        }
        if let Some(header) = update.header {
            let media_attachment = self.attachment_service.upload(header).await?;
            changeset = UpdateAccount {
                header_id: Some(media_attachment.id),
                ..changeset
            };
        }
        if let Some(locked) = update.locked {
            changeset = UpdateAccount {
                locked: Some(locked),
                ..changeset
            };
        }

        let updated_account: Account = self
            .db_pool
            .with_connection(|mut db_conn| {
                diesel::update(accounts::table.find(update.account_id))
                    .set(changeset)
                    .returning(Account::as_returning())
                    .get_result(&mut db_conn)
            })
            .await?;

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
