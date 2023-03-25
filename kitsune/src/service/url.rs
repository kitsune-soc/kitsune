use derive_builder::Builder;
use std::sync::Arc;
use uuid::Uuid;

/// Small "service" to centralise the creation of URLs
///
/// For some light deduplication purposes and to centralise the whole formatting story.
/// Allows for easier adjustments of URLs.
#[derive(Builder, Clone)]
pub struct UrlService {
    #[builder(setter(into))]
    scheme: Arc<str>,
    #[builder(setter(into))]
    domain: Arc<str>,
}

impl UrlService {
    #[must_use]
    pub fn builder() -> UrlServiceBuilder {
        UrlServiceBuilder::default()
    }

    #[must_use]
    pub fn base_url(&self) -> String {
        format!("{}://{}", self.scheme, self.domain)
    }

    #[must_use]
    pub fn default_avatar_url(&self) -> String {
        format!("{}/public/assets/default-avatar.png", self.base_url())
    }

    #[must_use]
    pub fn domain(&self) -> &str {
        &self.domain
    }

    #[must_use]
    pub fn favourite_url(&self, favourite_id: Uuid) -> String {
        format!("{}/favourites/{}", self.base_url(), favourite_id)
    }

    #[must_use]
    pub fn follow_url(&self, follow_id: Uuid) -> String {
        format!("{}/follows/{}", self.base_url(), follow_id)
    }

    #[must_use]
    pub fn media_url(&self, attachment_id: Uuid) -> String {
        format!("{}/media/{}", self.base_url(), attachment_id)
    }

    #[must_use]
    pub fn post_url(&self, post_id: Uuid) -> String {
        format!("{}/posts/{}", self.base_url(), post_id)
    }

    #[must_use]
    pub fn user_url(&self, username: &str) -> String {
        format!("{}/users/{}", self.base_url(), username)
    }
}
