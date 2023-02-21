use self::sealed::IntoMastodon;
use crate::{
    cache::{Cache, RedisCache},
    error::Result,
};
use derive_builder::Builder;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

mod sealed;

/// Cache Mastodon entities for 60 seconds
const CACHE_TTL: Duration = Duration::from_secs(60);

pub trait MapperMarker: IntoMastodon {}

impl<T> MapperMarker for T where T: IntoMastodon {}

#[derive(Builder, Clone)]
pub struct MastodonMapper {
    db_conn: DatabaseConnection,
    mastodon_cache: Arc<dyn Cache<Uuid, Value> + Send + Sync>,
}

impl MastodonMapper {
    #[must_use]
    pub fn builder() -> MastodonMapperBuilder {
        MastodonMapperBuilder::default()
    }

    /// Create a mapper with some defaults
    ///
    /// # Panics
    ///
    /// This should never panic.
    #[must_use]
    pub fn with_defaults(db_conn: DatabaseConnection, redis_conn: deadpool_redis::Pool) -> Self {
        let cache = Arc::new(RedisCache::new(
            redis_conn,
            "MASTODON-ENTITY-CACHE",
            CACHE_TTL,
        ));

        Self::builder()
            .db_conn(db_conn)
            .mastodon_cache(cache)
            .build()
            .unwrap()
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
