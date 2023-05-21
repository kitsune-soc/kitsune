use super::{handle_attachments, handle_mentions};
use crate::{
    cache::{ArcCache, CacheBackend},
    consts::USER_AGENT,
    error::{ApiError, Error, Result},
    sanitize::CleanHtmlExt,
    service::{
        federation_filter::FederationFilterService,
        search::{SearchBackend, SearchService},
    },
};
use async_recursion::async_recursion;
use autometrics::autometrics;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use http::HeaderValue;
use kitsune_db::{
    model::{
        account::{Account, AccountConflictChangeset, NewAccount, UpdateAccountMedia},
        media_attachment::NewMediaAttachment,
        post::{NewPost, Post, PostConflictChangeset, Visibility},
    },
    PgPool,
};
use kitsune_http_client::Client;
use kitsune_type::ap::{actor::Actor, Object};
use typed_builder::TypedBuilder;
use url::Url;
use uuid::{Timestamp, Uuid};

const MAX_FETCH_DEPTH: u32 = 50; // Maximum call depth of fetching new posts. Prevents unbounded recursion

#[derive(Clone, Debug, TypedBuilder)]
/// Options passed to the fetcher
pub struct FetchOptions<'a> {
    /// Refetch the ActivityPub entity
    ///
    /// This is mainly used to refresh possibly stale actors
    ///
    /// Default: false
    #[builder(default = false)]
    refetch: bool,

    /// URL of the ActivityPub entity
    url: &'a str,
}

impl<'a> From<&'a str> for FetchOptions<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().url(value).build()
    }
}

#[derive(Clone, TypedBuilder)]
pub struct Fetcher {
    #[builder(default =
        Client::builder()
            .default_header(
                "Accept",
                HeaderValue::from_static("application/activity+json"),
            )
            .unwrap()
            .user_agent(USER_AGENT)
            .unwrap()
            .build()
    )]
    client: Client,
    db_conn: PgPool,
    federation_filter: FederationFilterService,
    #[builder(setter(into))]
    search_service: SearchService,

    // Caches
    post_cache: ArcCache<str, Post>,
    user_cache: ArcCache<str, Account>,
}

impl Fetcher {
    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_actor(&self, opts: FetchOptions<'_>) -> Result<Account> {
        use kitsune_db::schema::{accounts, media_attachments};

        let mut db_conn = self.db_conn.get().await?;
        // Obviously we can't hit the cache nor the database if we wanna refetch the actor
        if !opts.refetch {
            if let Some(user) = self.user_cache.get(opts.url).await? {
                return Ok(user);
            }

            if let Some(user) = accounts::table
                .filter(accounts::url.eq(opts.url))
                .select(Account::columns())
                .first(&mut db_conn)
                .await
                .optional()?
            {
                return Ok(user);
            }
        }

        let url = Url::parse(opts.url)?;
        if !self.federation_filter.is_url_allowed(&url)? {
            return Err(ApiError::Unauthorised.into());
        }

        let mut actor: Actor = self.client.get(url.as_str()).await?.json().await?;
        actor.clean_html();

        let account: Account = db_conn
            .transaction(|tx| {
                #[allow(clippy::cast_sign_loss)]
                let uuid_timestamp = Timestamp::from_unix(
                    uuid::NoContext,
                    actor.published.unix_timestamp() as u64,
                    actor.published.nanosecond(),
                );

                async move {
                    let account = diesel::insert_into(accounts::table)
                        .values(NewAccount {
                            id: Uuid::new_v7(uuid_timestamp),
                            display_name: actor.name.as_deref(),
                            note: actor.subject.as_deref(),
                            username: actor.preferred_username.as_str(),
                            locked: actor.manually_approves_followers,
                            local: false,
                            domain: url.host_str().unwrap().into(),
                            actor_type: actor.r#type.into(),
                            url: Some(actor.id.as_str()),
                            featured_collection_url: actor.featured.as_deref(),
                            followers_url: Some(actor.followers.as_str()),
                            following_url: Some(actor.following.as_str()),
                            inbox_url: Some(actor.inbox.as_str()),
                            outbox_url: Some(actor.outbox.as_str()),
                            shared_inbox_url: actor
                                .endpoints
                                .and_then(|endpoints| endpoints.shared_inbox)
                                .as_deref(),
                            public_key_id: actor.public_key.id.as_str(),
                            public_key: actor.public_key.public_key_pem.as_str(),
                            created_at: Some(actor.published),
                        })
                        .on_conflict(accounts::url)
                        .do_update()
                        .set(AccountConflictChangeset {
                            display_name: actor.name.as_deref(),
                            note: actor.subject.as_deref(),
                            locked: actor.manually_approves_followers,
                            public_key_id: actor.public_key.id.as_str(),
                            public_key: actor.public_key.public_key_pem.as_str(),
                        })
                        .returning(Account::columns())
                        .get_result::<Account>(tx)
                        .await?;

                    let avatar_id = if let Some(icon) = actor.icon {
                        Some(
                            diesel::insert_into(media_attachments::table)
                                .values(NewMediaAttachment {
                                    id: Uuid::now_v7(),
                                    account_id: account.id,
                                    description: icon.name.as_deref(),
                                    content_type: icon.media_type.as_str(),
                                    blurhash: icon.blurhash.as_deref(),
                                    file_path: None,
                                    remote_url: Some(icon.url.as_str()),
                                })
                                .returning(media_attachments::id)
                                .get_result::<Uuid>(tx)
                                .await?,
                        )
                    } else {
                        None
                    };

                    let header_id = if let Some(image) = actor.image {
                        Some(
                            diesel::insert_into(media_attachments::table)
                                .values(NewMediaAttachment {
                                    id: Uuid::now_v7(),
                                    account_id: account.id,
                                    description: image.name.as_deref(),
                                    content_type: image.media_type.as_str(),
                                    blurhash: image.blurhash.as_deref(),
                                    file_path: None,
                                    remote_url: Some(image.url.as_str()),
                                })
                                .returning(media_attachments::id)
                                .get_result::<Uuid>(tx)
                                .await?,
                        )
                    } else {
                        None
                    };

                    let mut update_changeset = UpdateAccountMedia::default();
                    if let Some(avatar_id) = avatar_id {
                        update_changeset = UpdateAccountMedia {
                            avatar_id: Some(avatar_id),
                            ..update_changeset
                        };
                    }
                    if let Some(header_id) = header_id {
                        update_changeset = UpdateAccountMedia {
                            header_id: Some(header_id),
                            ..update_changeset
                        };
                    }

                    Ok::<_, Error>(
                        diesel::update(&account)
                            .set(update_changeset)
                            .returning(Account::columns())
                            .get_result(tx)
                            .await?,
                    )
                }
                .scope_boxed()
            })
            .await?;

        self.search_service
            .add_to_index(account.clone().into())
            .await?;

        Ok(account)
    }

    #[async_recursion]
    async fn fetch_object_inner(&self, url: &str, call_depth: u32) -> Result<Option<Post>> {
        use kitsune_db::schema::posts;

        if call_depth > MAX_FETCH_DEPTH {
            return Ok(None);
        }

        if !self.federation_filter.is_url_allowed(&Url::parse(url)?)? {
            return Err(ApiError::Unauthorised.into());
        }

        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(Some(post));
        }

        let mut db_conn = self.db_conn.get().await?;
        if let Some(post) = posts::table
            .filter(posts::url.eq(url))
            .select(Post::columns())
            .first(&mut db_conn)
            .await
            .optional()?
        {
            self.post_cache.set(url, &post).await?;
            return Ok(Some(post));
        }

        let url = Url::parse(url)?;
        let mut object: Object = self.client.get(url.as_str()).await?.json().await?;
        object.clean_html();

        let user = self.fetch_actor(object.attributed_to().into()).await?;
        let visibility = Visibility::from_activitypub(&user, &object).unwrap();

        #[allow(clippy::cast_sign_loss)]
        let uuid_timestamp = Timestamp::from_unix(
            uuid::NoContext,
            object.published.unix_timestamp() as u64,
            object.published.nanosecond(),
        );

        let in_reply_to_id = if let Some(in_reply_to) = object.in_reply_to {
            self.fetch_object_inner(&in_reply_to, call_depth + 1)
                .await?
                .map(|post| post.id)
        } else {
            None
        };

        let post = db_conn
            .transaction(|tx| {
                async move {
                    let new_post = diesel::insert_into(posts::table)
                        .values(NewPost {
                            id: Uuid::new_v7(uuid_timestamp),
                            account_id: user.id,
                            in_reply_to_id,
                            reposted_post_id: None,
                            subject: object.summary.as_deref(),
                            content: object.content.as_str(),
                            is_sensitive: object.sensitive,
                            visibility,
                            is_local: false,
                            url: object.id.as_str(),
                            created_at: Some(object.published),
                        })
                        .on_conflict(posts::url)
                        .do_update()
                        .set(PostConflictChangeset {
                            subject: object.summary.as_deref(),
                            content: object.content.as_str(),
                        })
                        .returning(Post::columns())
                        .get_result::<Post>(tx)
                        .await?;

                    handle_attachments(tx, &user, new_post.id, object.attachment).await?;
                    handle_mentions(tx, &user, new_post.id, &object.tag).await?;

                    Ok::<_, Error>(new_post)
                }
                .scope_boxed()
            })
            .await?;

        if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
            self.search_service
                .add_to_index(post.clone().into())
                .await?;
        }

        self.post_cache.set(&post.url, &post).await?;

        Ok(Some(post))
    }

    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_object(&self, url: &str) -> Result<Post> {
        self.fetch_object_inner(url, 0)
            .await
            .transpose()
            .expect("[Bug] Highest level fetch returned a `None`")
    }
}

/*#[cfg(test)]
mod test {
    use crate::{
        activitypub::Fetcher,
        cache::NoopCache,
        config::FederationFilterConfiguration,
        error::{ApiError, Error},
        service::{federation_filter::FederationFilterService, search::NoopSearchService},
    };
    use kitsune_db::entity::prelude::Accounts;
    use pretty_assertions::assert_eq;
    use sea_orm::EntityTrait;
    use std::sync::Arc;

    #[tokio::test]
    async fn fetch_actor() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::builder()
            .db_conn(db_conn)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_service(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let user = fetcher
            .fetch_actor("https://corteximplant.com/users/0x0".into())
            .await
            .expect("Fetch actor");

        assert_eq!(user.username, "0x0");
        assert_eq!(user.domain, "corteximplant.com");
        assert_eq!(user.url, Some("https://corteximplant.com/users/0x0".into()));
        assert_eq!(
            user.inbox_url,
            Some("https://corteximplant.com/users/0x0/inbox".into())
        );
    }

    #[tokio::test]
    async fn fetch_note() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::builder()
            .db_conn(db_conn.clone())
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: Vec::new(),
                })
                .unwrap(),
            )
            .search_service(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let note = fetcher
            .fetch_object("https://corteximplant.com/@0x0/109501674056556919")
            .await
            .expect("Fetch note");
        assert_eq!(
            note.url,
            "https://corteximplant.com/users/0x0/statuses/109501674056556919"
        );

        let author = Accounts::find_by_id(note.account_id)
            .one(&db_conn)
            .await
            .ok()
            .flatten()
            .expect("Get author");
        assert_eq!(author.username, "0x0");
        assert_eq!(
            author.url,
            Some("https://corteximplant.com/users/0x0".into())
        );
    }

    #[tokio::test]
    async fn federation_allow() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::builder()
            .db_conn(db_conn)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Allow {
                    domains: vec!["corteximplant.com".into()],
                })
                .unwrap(),
            )
            .search_service(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        assert!(matches!(
            fetcher.fetch_object("https://example.com/fakeobject").await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
        assert!(matches!(
            fetcher
                .fetch_object("https://other.badstuff.com/otherfake")
                .await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
        assert!(matches!(
            fetcher
                .fetch_object("https://corteximplant.com/@0x0/109501674056556919")
                .await,
            Ok(..)
        ));
    }

    #[tokio::test]
    async fn federation_deny() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::builder()
            .db_conn(db_conn)
            .federation_filter(
                FederationFilterService::new(&FederationFilterConfiguration::Deny {
                    domains: vec!["example.com".into(), "*.badstuff.com".into()],
                })
                .unwrap(),
            )
            .search_service(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        assert!(matches!(
            fetcher.fetch_object("https://example.com/fakeobject").await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
        assert!(matches!(
            fetcher
                .fetch_object("https://other.badstuff.com/otherfake")
                .await,
            Err(Error::Api(ApiError::Unauthorised))
        ));
    }
}*/
