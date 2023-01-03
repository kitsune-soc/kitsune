use crate::{
    cache::{Cache, RedisCache},
    consts::USER_AGENT,
    db::model::{
        account, media_attachment,
        post::{self, Visibility},
    },
    error::{Error, Result},
    sanitize::CleanHtmlExt,
};
use chrono::Utc;
use futures_util::FutureExt;
use http::{HeaderMap, HeaderValue};
use kitsune_type::ap::object::{Actor, Note};
use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    IntoActiveValue, QueryFilter, TransactionTrait,
};
use std::time::Duration;
use url::Url;
use uuid::{Timestamp, Uuid};

const CACHE_DURATION: Duration = Duration::from_secs(60); // 1 minute

#[derive(Clone)]
pub struct Fetcher<PC = RedisCache<str, post::Model>, UC = RedisCache<str, account::Model>> {
    client: Client,
    db_conn: DatabaseConnection,

    // Caches
    post_cache: PC,
    user_cache: UC,
}

impl Fetcher {
    #[must_use]
    pub fn with_redis_cache(db_conn: DatabaseConnection, redis_conn: deadpool_redis::Pool) -> Self {
        Self::new(
            db_conn,
            RedisCache::new(redis_conn.clone(), "fetcher-post", CACHE_DURATION),
            RedisCache::new(redis_conn, "fetcher-user", CACHE_DURATION),
        )
    }
}

impl<PC, UC> Fetcher<PC, UC>
where
    PC: Cache<str, post::Model>,
    UC: Cache<str, account::Model>,
{
    #[allow(clippy::missing_panics_doc)] // Invariants are covered. Won't panic.
    #[must_use]
    pub fn new(db_conn: DatabaseConnection, post_cache: PC, user_cache: UC) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "Accept",
            HeaderValue::from_static("application/activity+json"),
        );

        Self {
            client: Client::builder()
                .default_headers(default_headers)
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            db_conn,
            post_cache,
            user_cache,
        }
    }

    /// Fetch an ActivityPub actor
    ///
    /// # Panics
    ///
    /// - Panics if the URL doesn't contain a host section
    pub async fn fetch_actor(&self, url: &str) -> Result<account::Model> {
        if let Some(user) = self.user_cache.get(url).await? {
            return Ok(user);
        }

        if let Some(user) = account::Entity::find()
            .filter(account::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            return Ok(user);
        }

        let url = Url::parse(url)?;
        let mut actor: Actor = self.client.get(url.clone()).send().await?.json().await?;
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
                    let account = account::Model {
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
                        created_at: actor.rest.published,
                        updated_at: Utc::now(),
                    }
                    .into_active_model()
                    .insert(tx)
                    .await?;

                    let avatar_id = if let Some(icon) = actor.icon {
                        let media_attachment = media_attachment::Model {
                            id: Uuid::now_v7(),
                            account_id: account.id,
                            description: icon.name,
                            content_type: icon.media_type,
                            blurhash: icon.blurhash,
                            url: icon.url,
                            created_at: Utc::now(),
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;

                        Some(media_attachment.id)
                    } else {
                        None
                    };

                    let header_id = if let Some(image) = actor.image {
                        let media_attachment = media_attachment::Model {
                            id: Uuid::now_v7(),
                            account_id: account.id,
                            description: image.name,
                            content_type: image.media_type,
                            blurhash: image.blurhash,
                            url: image.url,
                            created_at: Utc::now(),
                        }
                        .into_active_model()
                        .insert(tx)
                        .await?;

                        Some(media_attachment.id)
                    } else {
                        None
                    };

                    account::ActiveModel {
                        id: ActiveValue::Set(account.id),
                        avatar_id: avatar_id.into_active_value(),
                        header_id: header_id.into_active_value(),
                        ..Default::default()
                    }
                    .update(tx)
                    .await
                    .map_err(Error::from)
                }
                .boxed()
            })
            .await
            .map_err(Error::from)
    }

    pub async fn fetch_note(&self, url: &str) -> Result<post::Model> {
        if let Some(post) = self.post_cache.get(url).await? {
            return Ok(post);
        }

        if let Some(post) = post::Entity::find()
            .filter(post::Column::Url.eq(url))
            .one(&self.db_conn)
            .await?
        {
            return Ok(post);
        }

        let mut note: Note = self.client.get(url).send().await?.json().await?;
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

        post::Model {
            id: Uuid::new_v7(uuid_timestamp),
            account_id: user.id,
            subject: note.subject,
            content: note.content,
            is_sensitive: note.rest.sensitive,
            visibility,
            url: note.rest.id,
            created_at: note.rest.published,
            updated_at: Utc::now(),
        }
        .into_active_model()
        .insert(&self.db_conn)
        .await
        .map_err(Error::from)
    }
}

#[cfg(test)]
mod test {
    use crate::{activitypub::Fetcher, cache::NoopCache, db::model::account};
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{Database, DatabaseConnection, ModelTrait};

    async fn prepare_db() -> DatabaseConnection {
        let db_conn = Database::connect("sqlite::memory:")
            .await
            .expect("Database connection");
        Migrator::up(&db_conn, None)
            .await
            .expect("Database migration");

        db_conn
    }

    #[tokio::test]
    async fn fetch_actor() {
        let db_conn = prepare_db().await;
        let fetcher = Fetcher::new(db_conn, NoopCache, NoopCache);

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
        let db_conn = prepare_db().await;
        let fetcher = Fetcher::new(db_conn.clone(), NoopCache, NoopCache);

        let note = fetcher
            .fetch_note("https://corteximplant.com/@0x0/109501674056556919")
            .await
            .expect("Fetch note");
        assert_eq!(
            note.url,
            "https://corteximplant.com/users/0x0/statuses/109501674056556919"
        );

        let author = note
            .find_related(account::Entity)
            .one(&db_conn)
            .await
            .ok()
            .flatten()
            .expect("Get author");
        assert_eq!(author.username, "0x0");
        assert_eq!(author.url, "https://corteximplant.com/users/0x0");
    }
}
