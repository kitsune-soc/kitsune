#[macro_use]
extern crate tracing;

use self::sealed::{IntoMastodon, MapperState};
use kitsune_cache::{ArcCache, CacheBackend};
use kitsune_db::PgPool;
use kitsune_embed::Client as EmbedClient;
use kitsune_error::Result;
use kitsune_service::attachment::AttachmentService;
use kitsune_url::UrlService;
use serde::Deserialize;
use simd_json::OwnedValue;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

mod sealed;

pub trait MapperMarker: IntoMastodon {}

impl<T> MapperMarker for T where T: IntoMastodon {}

#[derive(Clone, TypedBuilder)]
pub struct MastodonMapper {
    attachment_service: AttachmentService,
    db_pool: PgPool,
    embed_client: Option<EmbedClient>,
    mastodon_cache: ArcCache<Uuid, OwnedValue>,
    url_service: UrlService,
}

impl MastodonMapper {
    /// Return a reference to a mapper state
    ///
    /// Passed down to the concrete mapping implementations
    fn mapper_state(&self) -> MapperState<'_> {
        MapperState {
            attachment_service: &self.attachment_service,
            db_pool: &self.db_pool,
            embed_client: self.embed_client.as_ref(),
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
                .map(<T::Output as Deserialize>::deserialize)
            {
                Some(Ok(entity)) => return Ok(entity),
                Some(Err(err)) => error!(error = %err, "Failed to deserialise entity from cache"),
                None => (),
            }
        }

        let entity = input.into_mastodon(self.mapper_state()).await?;
        if let Some(id) = input_id {
            let entity = simd_json::serde::to_owned_value(&entity).unwrap();
            self.mastodon_cache.set(&id, &entity).await?;
        }

        Ok(entity)
    }
}
