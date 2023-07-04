use smol_str::SmolStr;
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

/// Small "service" to centralise the creation of URLs
///
/// For some light deduplication purposes and to centralise the whole formatting story.
/// Allows for easier adjustments of URLs.
#[derive(Clone, TypedBuilder)]
pub struct UrlService {
    #[builder(setter(into))]
    scheme: SmolStr,
    #[builder(setter(into))]
    domain: SmolStr,
}

impl UrlService {
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("{}://{}", self.scheme, self.domain)
    }

    #[must_use]
    pub fn default_avatar_url(&self) -> String {
        format!("{}/public/assets/default-avatar.png", self.base_url())
    }

    #[must_use]
    pub fn default_header_url(&self) -> String {
        format!("{}/public/assets/default-header.png", self.base_url())
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
    pub fn followers_url(&self, user_id: Uuid) -> String {
        format!("{}/followers", self.user_url(user_id))
    }

    #[must_use]
    pub fn following_url(&self, user_id: Uuid) -> String {
        format!("{}/following", self.user_url(user_id))
    }

    #[must_use]
    pub fn inbox_url(&self, user_id: Uuid) -> String {
        format!("{}/inbox", self.user_url(user_id))
    }

    #[must_use]
    pub fn outbox_url(&self, user_id: Uuid) -> String {
        format!("{}/outbox", self.user_url(user_id))
    }

    #[must_use]
    pub fn media_url(&self, attachment_id: Uuid) -> String {
        format!("{}/media/{}", self.base_url(), attachment_id)
    }

    #[must_use]
    pub fn oidc_redirect_uri(&self) -> String {
        format!("{}/oidc/callback", self.base_url())
    }

    #[must_use]
    pub fn post_url(&self, post_id: Uuid) -> String {
        format!("{}/posts/{}", self.base_url(), post_id)
    }

    #[must_use]
    pub fn public_key_id(&self, user_id: Uuid) -> String {
        format!("{}#main-key", self.user_url(user_id))
    }

    #[must_use]
    pub fn user_url(&self, user_id: Uuid) -> String {
        format!("{}/users/{user_id}", self.base_url())
    }
}
