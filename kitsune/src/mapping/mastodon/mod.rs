use self::sealed::IntoMastodon;
use crate::{
    cache::{Cache, RedisCache},
    error::Result,
    event::{status::EventType, StatusEventConsumer},
};
use derive_builder::Builder;
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

mod sealed;

/// Cache Mastodon entities for 60 seconds
const CACHE_TTL: Duration = Duration::from_secs(60);

pub trait MapperMarker: IntoMastodon {}

impl<T> MapperMarker for T where T: IntoMastodon {}

#[derive(Builder)]
#[builder(pattern = "owned")]
struct CacheInvalidationActor {
    cache: Arc<dyn Cache<Uuid, Value> + Send + Sync>,
    event_consumer: StatusEventConsumer,
}

impl CacheInvalidationActor {
    #[must_use]
    pub fn builder() -> CacheInvalidationActorBuilder {
        CacheInvalidationActorBuilder::default()
    }

    async fn run(mut self) {
        loop {
            while let Some(event) = self.event_consumer.next().await {
                let event = match event {
                    Ok(event) => event,
                    Err(err) => {
                        error!(error = %err, "Failed to receive status event");
                        continue;
                    }
                };

                if matches!(event.r#type, EventType::Delete | EventType::Update) {
                    if let Err(err) = self.cache.delete(&event.status_id).await {
                        error!(error = %err, "Failed to remove entry from cache");
                    }
                }
            }
        }
    }

    pub fn spawn(self) {
        tokio::spawn(self.run());
    }
}

#[derive(Clone)]
pub struct MastodonMapper {
    db_conn: DatabaseConnection,
    mastodon_cache: Arc<dyn Cache<Uuid, Value> + Send + Sync>,
}

impl MastodonMapper {
    /// Create a new Mastodon mapper
    ///
    /// # Panics
    ///
    /// This should never panic
    pub fn new(
        db_conn: DatabaseConnection,
        mastodon_cache: Arc<dyn Cache<Uuid, Value> + Send + Sync>,
        status_event_consumer: StatusEventConsumer,
    ) -> Self {
        CacheInvalidationActor::builder()
            .cache(mastodon_cache.clone())
            .event_consumer(status_event_consumer)
            .build()
            .unwrap()
            .spawn();

        Self {
            db_conn,
            mastodon_cache,
        }
    }

    /// Create a mapper with some defaults
    ///
    /// # Panics
    ///
    /// This should never panic.
    #[must_use]
    pub fn with_defaults(
        db_conn: DatabaseConnection,
        redis_conn: deadpool_redis::Pool,
        event_consumer: StatusEventConsumer,
    ) -> Self {
        let cache = Arc::new(RedisCache::new(
            redis_conn,
            "MASTODON-ENTITY-CACHE",
            CACHE_TTL,
        ));

        Self::new(db_conn, cache, event_consumer)
    }

    /// Map some input into a Mastodon API entity
    ///
    /// # Panics
    ///
    /// This should never panic.
    pub async fn map<T>(&self, input: T) -> Result<T::Output>
    where
        T: MapperMarker,
    {
        let input_id = input.id();

        if let Some(id) = input_id {
            match self
                .mastodon_cache
                .get(&id)
                .await?
                .map(serde_json::from_value)
            {
                Some(Ok(entity)) => return Ok(entity),
                Some(Err(err)) => error!(error = %err, "Failed to deserialise entity from cache"),
                None => (),
            }
        }

        let entity = input.into_mastodon(&self.db_conn).await?;
        if let Some(id) = input_id {
            let entity = serde_json::to_value(entity.clone()).unwrap();
            self.mastodon_cache.set(&id, &entity).await?;
        }

        Ok(entity)
    }
}
