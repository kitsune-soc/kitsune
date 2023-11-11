use crate::oauth2::{OAuth2Service, OAuthEndpoint};
use axum_extra::extract::cookie;
use kitsune_core::{
    activitypub::Fetcher,
    event::PostEventEmitter,
    service::{
        account::AccountService, attachment::AttachmentService, custom_emoji::CustomEmojiService,
        federation_filter::FederationFilterService, instance::InstanceService, job::JobService,
        notification::NotificationService, post::PostService, search::SearchService,
        timeline::TimelineService, url::UrlService, user::UserService,
    },
    state::{EventEmitter, Service as CoreServiceState, State as CoreState},
};
use kitsune_db::PgPool;
use kitsune_embed::Client as EmbedClient;
use std::{ops::Deref, sync::Arc};

#[cfg(feature = "mastodon-api")]
use kitsune_core::mapping::MastodonMapper;

#[cfg(feature = "oidc")]
use kitsune_oidc::OidcService;

#[macro_export]
macro_rules! impl_from_ref {
    ($source:path; [ $($target:path => $extract_impl:expr),+ ]) => {
        $(
            impl ::axum::extract::FromRef<$source> for $target {
                fn from_ref(input: &$source) -> Self {
                    #[allow(clippy::redundant_closure_call)]
                    ($extract_impl)(input)
                }
            }
        )+
    };
}

impl_from_ref! {
    Zustand;
    [
        PgPool => |input: &Zustand| input.db_pool().clone()
    ]
}

#[cfg(feature = "mastodon-api")]
impl_from_ref! {
    Zustand;
    [
        kitsune_core::mapping::MastodonMapper => |input: &Zustand| input.mastodon_mapper().clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        AccountService => |input: &Zustand| input.core.service.account.clone(),
        AttachmentService => |input: &Zustand| input.core.service.attachment.clone(),
        CustomEmojiService => |input: &Zustand| input.core.service.custom_emoji.clone(),
        FederationFilterService => |input: &Zustand| input.core.service.federation_filter.clone(),
        JobService => |input: &Zustand| input.core.service.job.clone(),
        NotificationService => |input: &Zustand| input.core.service.notification.clone(),
        PostService => |input: &Zustand| input.core.service.post.clone(),
        SearchService => |input: &Zustand| input.core.service.search.clone(),
        InstanceService => |input: &Zustand| input.core.service.instance.clone(),
        TimelineService => |input: &Zustand| input.core.service.timeline.clone(),
        UrlService => |input: &Zustand| input.core.service.url.clone(),
        UserService => |input: &Zustand| input.core.service.user.clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        PostEventEmitter => |input: &Zustand| input.event_emitter().post.clone()
    ]
}

#[derive(Clone)]
pub struct SessionConfig {
    pub cookie_key: cookie::Key,
    pub flash_config: axum_flash::Config,
}

impl SessionConfig {
    /// Randomly generates the keys for the cookie jars
    #[must_use]
    pub fn generate() -> Self {
        let cookie_key = cookie::Key::generate();
        #[allow(unused_mut)]
        let mut flash_config = axum_flash::Config::new(axum_flash::Key::generate());

        #[cfg(debug_assertions)]
        {
            flash_config = flash_config.use_secure_cookies(false);
        }

        Self {
            cookie_key,
            flash_config,
        }
    }
}

impl_from_ref! {
    Zustand;
    [
        cookie::Key => |input: &Zustand| input.session_config.cookie_key.clone(),
        axum_flash::Config => |input: &Zustand| input.session_config.flash_config.clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        CoreState => |input: &Zustand| input.core.clone(),
        OAuth2Service => |input: &Zustand| input.oauth2.clone(),
        OAuthEndpoint => |input: &Zustand| input.oauth_endpoint.clone(),
        SessionConfig => |input: &Zustand| input.session_config.clone()
    ]
}

#[cfg(feature = "oidc")]
impl_from_ref! {
    Zustand;
    [
        Option<OidcService> => |input: &Zustand| input.oidc.clone()
    ]
}

pub struct ZustandInner {
    pub core: CoreState,
    pub oauth2: OAuth2Service,
    pub oauth_endpoint: OAuthEndpoint,
    #[cfg(feature = "oidc")]
    pub oidc: Option<OidcService>,
    pub session_config: SessionConfig,
}

#[derive(Clone)]
pub struct Zustand(Arc<ZustandInner>);

impl Deref for Zustand {
    type Target = ZustandInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ZustandInner> for Zustand {
    fn from(value: ZustandInner) -> Self {
        Self(Arc::new(value))
    }
}

impl Zustand {
    #[inline]
    #[must_use]
    pub fn db_pool(&self) -> &PgPool {
        &self.core.db_pool
    }

    #[inline]
    #[must_use]
    pub fn embed_client(&self) -> Option<&EmbedClient> {
        self.core.embed_client.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn event_emitter(&self) -> &EventEmitter {
        &self.core.event_emitter
    }

    #[inline]
    #[must_use]
    pub fn fetcher(&self) -> &Fetcher {
        &self.core.fetcher
    }

    #[inline]
    #[must_use]
    #[cfg(feature = "mastodon-api")]
    pub fn mastodon_mapper(&self) -> &MastodonMapper {
        &self.core.mastodon_mapper
    }

    #[inline]
    #[must_use]
    pub fn service(&self) -> &CoreServiceState {
        &self.core.service
    }
}
