use self::sealed::{IntoMastodon, MapperState};
use crate::{
    cache::{ArcCache, CacheBackend},
    error::Result,
    event::{post::EventType, PostEventConsumer},
    service::{attachment::AttachmentService, url::UrlService},
};
use derive_builder::Builder;
use futures_util::StreamExt;
use kitsune_db::PgPool;
use serde_json::Value;
use typed_builder::TypedBuilder;
use uuid::Uuid;

mod sealed;

pub trait MapperMarker: IntoMastodon {}

impl<T> MapperMarker for T where T: IntoMastodon {}

#[derive(TypedBuilder)]
struct CacheInvalidationActor {
    cache: ArcCache<Uuid, Value>,
    event_consumer: PostEventConsumer,
}

impl CacheInvalidationActor {
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
                    if let Err(err) = self.cache.delete(&event.post_id).await {
                        error!(error = %err, "Failed to remove entry from cache");
                    }
                }
            }

            if let Err(err) = self.event_consumer.reconnect().await {
                error!(error = %err, "Failed to reconnect to event source");
            }
        }
    }

    pub fn spawn(self) {
        tokio::spawn(self.run());
    }
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
#[allow(clippy::used_underscore_binding)]
pub struct MastodonMapper {
    #[builder(
        field(
            type = "Option<PostEventConsumer>",
            build = "CacheInvalidationActor::builder()
                        .cache(
                            self.mastodon_cache
                                .clone()
                                .ok_or(MastodonMapperBuilderError::UninitializedField(\"mastodon_cache\"))?
                        )
                        .event_consumer(
                            self._cache_invalidator
                                .ok_or(MastodonMapperBuilderError::UninitializedField(\"cache_invalidator\"))?
                        )
                        .build()
                        .spawn();",
        ),
        setter(name = "cache_invalidator", strip_option)
    )]
    _cache_invalidator: (),
    attachment_service: AttachmentService,
    db_conn: PgPool,
    mastodon_cache: ArcCache<Uuid, Value>,
    url_service: UrlService,
}

impl MastodonMapper {
    #[must_use]
    pub fn builder() -> MastodonMapperBuilder {
        MastodonMapperBuilder::default()
    }

    /// Return a reference to a mapper state
    ///
    /// Passed down to the concrete mapping implementations
    fn mapper_state(&self) -> MapperState<'_> {
        MapperState {
            attachment_service: &self.attachment_service,
            db_conn: &self.db_conn,
            url_service: &self.url_service,
        }
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

        let entity = input.into_mastodon(self.mapper_state()).await?;
        if let Some(id) = input_id {
            let entity = serde_json::to_value(entity.clone()).unwrap();
            self.mastodon_cache.set(&id, &entity).await?;
        }

        Ok(entity)
    }
}
