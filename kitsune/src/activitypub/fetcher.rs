use crate::{
    cache::{Cache, RedisCache},
    consts::USER_AGENT,
    error::{Error, Result},
    sanitize::CleanHtmlExt,
    search::{GrpcSearchService, SearchService},
};
use async_recursion::async_recursion;
use autometrics::autometrics;
use chrono::Utc;
use futures_util::FutureExt;
use http::HeaderValue;
use kitsune_db::{
    custom::Visibility,
    entity::{accounts, media_attachments, posts},
};
use kitsune_http_client::Client;
use kitsune_type::ap::object::{Actor, Note};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    IntoActiveValue, QueryFilter, TransactionTrait,
};
use std::time::Duration;
use url::Url;
use uuid::{Timestamp, Uuid};

const CACHE_DURATION: Duration = Duration::from_secs(60); // 1 minute
const MAX_FETCH_DEPTH: u32 = 100; // Maximum call depth of fetching new posts. Prevents unbounded recursion

#[derive(Clone)]
pub struct Fetcher<
    S = GrpcSearchService,
    PC = RedisCache<str, posts::Model>,
    UC = RedisCache<str, accounts::Model>,
> {
    client: Client,
    db_conn: DatabaseConnection,
    search_service: S,

    // Caches
    post_cache: PC,
    user_cache: UC,
}

impl Fetcher {
    #[must_use]
    pub fn with_defaults(
        db_conn: DatabaseConnection,
        search_service: GrpcSearchService,
        redis_conn: deadpool_redis::Pool,
    ) -> Self {
        Self::new(
            db_conn,
            search_service,
            RedisCache::new(redis_conn.clone(), "fetcher-post", CACHE_DURATION),
            RedisCache::new(redis_conn, "fetcher-user", CACHE_DURATION),
        )
    }
}

impl<S, PC, UC> Fetcher<S, PC, UC>
where
    S: SearchService,
    PC: Cache<str, posts::Model>,
    UC: Cache<str, accounts::Model>,
{
    #[allow(clippy::missing_panics_doc)] // Invariants are covered. Won't panic.
    #[must_use]
    pub fn new(
        db_conn: DatabaseConnection,
        search_service: S,
        post_cache: PC,
        user_cache: UC,
    ) -> Self {
        Self {
            client: Client::builder()
                .default_header(
                    "Accept",
                    HeaderValue::from_static("application/activity+json"),
                )
                .unwrap()
                .user_agent(USER_AGENT)
                .unwrap()
                .build(),
            db_conn,
            search_service,
            post_cache,
            user_cache,
        }
    }

    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_actor(&self, url: &str) -> Result<accounts::Model> {
        if let Some(user) = self.user_cache.get(url).await? {
            return Ok(user);
        }

        if let Some(user) = accounts::Entity::find()
            .filter(accounts::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            return Ok(user);
        }

        let mut search_service = self.search_service.clone();
        let url = Url::parse(url)?;
        let mut actor: Actor = self.client.get(url.as_str()).await?.json().await?;
        actor.clean_html();

        self.db_conn
            .transaction(|tx| {
                #[allow(clippy::cast_sign_loss)]
                let uuid_timestamp = Timestamp::from_unix(
                    uuid::NoContext,
                    actor.rest.published.timestamp() as u64,
                    actor.rest.published.timestamp_subsec_nanos(),
                );

                async move {
                    let account = accounts::Model {
                        id: Uuid::new_v7(uuid_timestamp),
                        avatar_id: None,
                        header_id: None,
                        display_name: actor.name,
                        note: actor.subject,
                        username: actor.preferred_username,
                        locked: actor.manually_approves_followers,
                        domain: Some(url.host_str().unwrap().into()),
                        url: actor.rest.id,
                        followers_url: actor.followers,
                        inbox_url: actor.inbox,
                        public_key: actor.public_key.public_key_pem,
                        created_at: actor.rest.published.into(),
                        updated_at: Utc::now().into(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    let avatar_id = if let Some(icon) = actor.icon {
                        let media_attachment = media_attachments::Model {
                            id: Uuid::now_v7(),
                            account_id: account.id,
                            description: icon.name,
                            content_type: icon.media_type,
                            blurhash: icon.blurhash,
                            url: icon.url,
                            created_at: Utc::now().into(),
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;

                        Some(media_attachment.id)
                    } else {
                        None
                    };

                    let header_id = if let Some(image) = actor.image {
                        let media_attachment = media_attachments::Model {
                            id: Uuid::now_v7(),
                            account_id: account.id,
                            description: image.name,
                            content_type: image.media_type,
                            blurhash: image.blurhash,
                            url: image.url,
                            created_at: Utc::now().into(),
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;

                        Some(media_attachment.id)
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
                    search_service.add_to_index(account.clone()).await?;

                    Ok(account)
                }
                .boxed()
            })
            .await
            .map_err(Error::from)
    }

    #[async_recursion(?Send)]
    async fn fetch_note_inner(&self, url: &str, call_depth: u32) -> Result<Option<posts::Model>> {
        if call_depth > MAX_FETCH_DEPTH {
            return Ok(None);
        }

        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(Some(post));
        }

        if let Some(post) = posts::Entity::find()
            .filter(posts::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            return Ok(Some(post));
        }

        let url = Url::parse(url)?;
        let mut note: Note = self.client.get(url.as_str()).await?.json().await?;
        note.clean_html();

        let user = self
            .fetch_actor(note.rest.attributed_to().ok_or(Error::MalformedApObject)?)
            .await?;
        let visibility = Visibility::from_activitypub(&user, &note);

        #[allow(clippy::cast_sign_loss)]
        let uuid_timestamp = Timestamp::from_unix(
            uuid::NoContext,
            note.rest.published.timestamp() as u64,
            note.rest.published.timestamp_subsec_nanos(),
        );

        let in_reply_to_id = if let Some(in_reply_to) = note.rest.in_reply_to {
            self.fetch_note_inner(&in_reply_to, call_depth + 1)
                .await?
                .map(|post| post.id)
        } else {
            None
        };

        let post = posts::Model {
            id: Uuid::new_v7(uuid_timestamp),
            account_id: user.id,
            in_reply_to_id,
            subject: note.subject,
            content: note.content,
            is_sensitive: note.rest.sensitive,
            visibility,
            url: note.rest.id,
            created_at: note.rest.published.into(),
            updated_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await?;

        if post.visibility == Visibility::Public || post.visibility == Visibility::Unlisted {
            self.search_service
                .clone()
                .add_to_index(post.clone())
                .await?;
        }

        Ok(Some(post))
    }

    #[instrument(skip(self))]
    #[autometrics(track_concurrency)]
    pub async fn fetch_note(&self, url: &str) -> Result<posts::Model> {
        self.fetch_note_inner(url, 0)
            .await
            .transpose()
            .expect("[Bug] Highest level fetch returned a `None`")
    }
}

#[cfg(test)]
mod test {
    use crate::{activitypub::Fetcher, cache::NoopCache, search::NoopSearchService};
    use kitsune_db::entity::accounts;
    use pretty_assertions::assert_eq;
    use sea_orm::ModelTrait;

    #[tokio::test]
    async fn fetch_actor() {
        let db_conn = kitsune_db::connect("sqlite::memory:").await.unwrap();
        let fetcher = Fetcher::new(db_conn, NoopSearchService, NoopCache, NoopCache);

        let user = fetcher
            .fetch_actor("https://corteximplant.com/users/0x0")
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
        let fetcher = Fetcher::new(db_conn.clone(), NoopSearchService, NoopCache, NoopCache);

        let note = fetcher
            .fetch_note("https://corteximplant.com/@0x0/109501674056556919")
            .await
            .expect("Fetch note");
        assert_eq!(
            note.url,
            "https://corteximplant.com/users/0x0/statuses/109501674056556919"
        );

        let author = note
            .find_related(accounts::Entity)
            .one(&db_conn)
            .await
            .ok()
            .flatten()
            .expect("Get author");
        assert_eq!(author.username, "0x0");
        assert_eq!(author.url, "https://corteximplant.com/users/0x0");
    }
}
