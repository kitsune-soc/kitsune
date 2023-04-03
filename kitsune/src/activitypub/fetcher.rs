use crate::{
    cache::{ArcCache, CacheBackend, RedisCache},
    consts::USER_AGENT,
    error::{Error, Result},
    sanitize::CleanHtmlExt,
    service::search::{GrpcSearchService, SearchBackend, SearchService},
};
use async_recursion::async_recursion;
use autometrics::autometrics;
use chrono::Utc;
use futures_util::FutureExt;
use http::HeaderValue;
use kitsune_db::{
    custom::Visibility,
    entity::{
        accounts, media_attachments, posts,
        prelude::{Accounts, MediaAttachments, Posts},
    },
};
use kitsune_http_client::Client;
use kitsune_type::ap::{object::Actor, Object};
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, IntoActiveValue, QueryFilter, TransactionTrait,
};
use std::{sync::Arc, time::Duration};
use typed_builder::TypedBuilder;
use url::Url;
use uuid::{Timestamp, Uuid};

use super::{handle_attachments, handle_mentions};

const CACHE_DURATION: Duration = Duration::from_secs(60); // 1 minute
const MAX_FETCH_DEPTH: u32 = 100; // Maximum call depth of fetching new posts. Prevents unbounded recursion

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
    db_conn: DatabaseConnection,
    #[builder(setter(into))]
    search_service: SearchService,

    // Caches
    post_cache: ArcCache<str, posts::Model>,
    user_cache: ArcCache<str, accounts::Model>,
}

impl Fetcher {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn with_defaults(
        db_conn: DatabaseConnection,
        search_service: GrpcSearchService,
        redis_conn: deadpool_redis::Pool,
    ) -> Self {
        Self::builder()
            .db_conn(db_conn)
            .search_service(search_service)
            .post_cache(Arc::new(
                RedisCache::new(redis_conn.clone(), "fetcher-post", CACHE_DURATION).into(),
            ))
            .user_cache(Arc::new(
                RedisCache::new(redis_conn, "fetcher-user", CACHE_DURATION).into(),
            ))
            .build()
    }

    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_actor(&self, opts: FetchOptions<'_>) -> Result<accounts::Model> {
        // Obviously we can't hit the cache nor the database if we wanna refetch the actor
        if !opts.refetch {
            if let Some(user) = self.user_cache.get(opts.url).await? {
                return Ok(user);
            }

            if let Some(user) = Accounts::find()
                .filter(accounts::Column::Url.eq(opts.url))
                .one(&self.db_conn)
                .await?
            {
                return Ok(user);
            }
        }

        let url = Url::parse(opts.url)?;
        let mut actor: Actor = self.client.get(url.as_str()).await?.json().await?;
        actor.clean_html();

        let account = self
            .db_conn
            .transaction(|tx| {
                #[allow(clippy::cast_sign_loss)]
                let uuid_timestamp = Timestamp::from_unix(
                    uuid::NoContext,
                    actor.rest.published.timestamp() as u64,
                    actor.rest.published.timestamp_subsec_nanos(),
                );

                async move {
                    let account = Accounts::insert(
                        accounts::Model {
                            id: Uuid::new_v7(uuid_timestamp),
                            avatar_id: None,
                            header_id: None,
                            display_name: actor.name,
                            note: actor.subject,
                            username: actor.preferred_username,
                            locked: actor.manually_approves_followers,
                            local: false,
                            domain: Some(url.host_str().unwrap().into()),
                            url: actor.rest.id,
                            followers_url: actor.followers,
                            inbox_url: actor.inbox,
                            public_key: actor.public_key.public_key_pem,
                            created_at: actor.rest.published.into(),
                            updated_at: Utc::now().into(),
                        }
                        .into_active_model(),
                    )
                    .on_conflict(
                        OnConflict::column(accounts::Column::Url)
                            .update_columns([
                                accounts::Column::DisplayName,
                                accounts::Column::Note,
                                accounts::Column::Locked,
                                accounts::Column::PublicKey,
                                accounts::Column::UpdatedAt,
                            ])
                            .clone(),
                    )
                    .exec_with_returning(tx)
                    .await?;

                    let avatar_id = if let Some(icon) = actor.icon {
                        let insert_result = MediaAttachments::insert(
                            media_attachments::Model {
                                id: Uuid::now_v7(),
                                account_id: account.id,
                                description: icon.name,
                                content_type: icon.media_type,
                                blurhash: icon.blurhash,
                                file_path: None,
                                remote_url: Some(icon.url),
                                created_at: Utc::now().into(),
                                updated_at: Utc::now().into(),
                            }
                            .into_active_model(),
                        )
                        .exec(tx)
                        .await?;

                        Some(insert_result.last_insert_id)
                    } else {
                        None
                    };

                    let header_id = if let Some(image) = actor.image {
                        let insert_result = MediaAttachments::insert(
                            media_attachments::Model {
                                id: Uuid::now_v7(),
                                account_id: account.id,
                                description: image.name,
                                content_type: image.media_type,
                                blurhash: image.blurhash,
                                file_path: None,
                                remote_url: Some(image.url),
                                created_at: Utc::now().into(),
                                updated_at: Utc::now().into(),
                            }
                            .into_active_model(),
                        )
                        .exec(tx)
                        .await?;

                        Some(insert_result.last_insert_id)
                    } else {
                        None
                    };

                    let account = accounts::ActiveModel {
                        id: ActiveValue::Set(account.id),
                        avatar_id: avatar_id.into_active_value(),
                        header_id: header_id.into_active_value(),
                        ..Default::default()
                    }
                    .update(tx)
                    .await?;

                    Ok::<_, Error>(account)
                }
                .boxed()
            })
            .await?;
        self.search_service
            .add_to_index(account.clone().into())
            .await?;

        Ok(account)
    }

    #[async_recursion]
    async fn fetch_object_inner(&self, url: &str, call_depth: u32) -> Result<Option<posts::Model>> {
        if call_depth > MAX_FETCH_DEPTH {
            return Ok(None);
        }

        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(Some(post));
        }

        if let Some(post) = Posts::find()
            .filter(posts::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            self.post_cache.set(url, &post).await?;
            return Ok(Some(post));
        }

        let url = Url::parse(url)?;
        let mut object: Object = self.client.get(url.as_str()).await?.json().await?;
        object.clean_html();

        let user = self
            .fetch_actor(
                object
                    .rest
                    .attributed_to()
                    .map(FetchOptions::from)
                    .ok_or(Error::MalformedApObject)?,
            )
            .await?;
        let visibility = Visibility::from_activitypub(&user, &object);

        #[allow(clippy::cast_sign_loss)]
        let uuid_timestamp = Timestamp::from_unix(
            uuid::NoContext,
            object.rest.published.timestamp() as u64,
            object.rest.published.timestamp_subsec_nanos(),
        );

        let in_reply_to_id = if let Some(in_reply_to) = object.rest.in_reply_to {
            self.fetch_object_inner(&in_reply_to, call_depth + 1)
                .await?
                .map(|post| post.id)
        } else {
            None
        };

        let post = self
            .db_conn
            .transaction(|tx| {
                async move {
                    let new_post = Posts::insert(
                        posts::Model {
                            id: Uuid::new_v7(uuid_timestamp),
                            account_id: user.id,
                            in_reply_to_id,
                            reposted_post_id: None,
                            subject: object.summary,
                            content: object.content,
                            is_sensitive: object.rest.sensitive,
                            visibility,
                            is_local: false,
                            url: object.rest.id,
                            created_at: object.rest.published.into(),
                            updated_at: Utc::now().into(),
                        }
                        .into_active_model(),
                    )
                    .on_conflict(
                        OnConflict::column(posts::Column::Url)
                            .update_columns([posts::Column::Content, posts::Column::Subject])
                            .clone(),
                    )
                    .exec_with_returning(tx)
                    .await?;

                    handle_attachments(tx, &user, new_post.id, object.attachment).await?;
                    handle_mentions(tx, &user, new_post.id, &object.tag).await?;

                    Ok::<_, Error>(new_post)
                }
                .boxed()
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
    pub async fn fetch_object(&self, url: &str) -> Result<posts::Model> {
        self.fetch_object_inner(url, 0)
            .await
            .transpose()
            .expect("[Bug] Highest level fetch returned a `None`")
    }
}

#[cfg(test)]
mod test {
    use crate::{activitypub::Fetcher, cache::NoopCache, service::search::NoopSearchService};
    use kitsune_db::entity::prelude::Accounts;
    use pretty_assertions::assert_eq;
    use sea_orm::EntityTrait;
    use std::sync::Arc;

    #[tokio::test]
    async fn fetch_actor() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::builder()
            .db_conn(db_conn)
            .search_service(NoopSearchService)
            .post_cache(Arc::new(NoopCache.into()))
            .user_cache(Arc::new(NoopCache.into()))
            .build();

        let user = fetcher
            .fetch_actor("https://corteximplant.com/users/0x0".into())
            .await
            .expect("Fetch actor");

        assert_eq!(user.username, "0x0");
        assert_eq!(user.domain, Some("corteximplant.com".into()));
        assert_eq!(user.url, "https://corteximplant.com/users/0x0");
        assert_eq!(user.inbox_url, "https://corteximplant.com/users/0x0/inbox");
    }

    #[tokio::test]
    async fn fetch_note() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::builder()
            .db_conn(db_conn.clone())
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
        assert_eq!(author.url, "https://corteximplant.com/users/0x0");
    }
}
