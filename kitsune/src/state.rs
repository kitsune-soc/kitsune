use crate::oauth2::{OAuth2Service, OAuthEndpoint};
use axum::extract::FromRef;
use axum_extra::extract::cookie;
use kitsune_core::{
    event::PostEventEmitter,
    service::{
        account::AccountService, attachment::AttachmentService,
        federation_filter::FederationFilterService, instance::InstanceService, job::JobService,
        notification::NotificationService, post::PostService, timeline::TimelineService,
        url::UrlService, user::UserService,
    },
    state::State as CoreState,
};
use kitsune_db::PgPool;
use kitsune_search::SearchService;

#[cfg(feature = "oidc")]
use crate::oidc::OidcService;

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
    AppState;
    [
        PgPool => |input: &AppState| input.core.db_pool.clone()
    ]
}

#[cfg(feature = "mastodon-api")]
impl_from_ref! {
    AppState;
    [
        kitsune_core::mapping::MastodonMapper => |input: &AppState| input.core.mastodon_mapper.clone()
    ]
}

impl_from_ref! {
    AppState;
    [
        AccountService => |input: &AppState| input.core.service.account.clone(),
        AttachmentService => |input: &AppState| input.core.service.attachment.clone(),
        FederationFilterService => |input: &AppState| input.core.service.federation_filter.clone(),
        JobService => |input: &AppState| input.core.service.job.clone(),
        NotificationService => |input: &AppState| input.core.service.notification.clone(),
        PostService => |input: &AppState| input.core.service.post.clone(),
        SearchService => |input: &AppState| input.core.service.search.clone(),
        InstanceService => |input: &AppState| input.core.service.instance.clone(),
        TimelineService => |input: &AppState| input.core.service.timeline.clone(),
        UrlService => |input: &AppState| input.core.service.url.clone(),
        UserService => |input: &AppState| input.core.service.user.clone()
    ]
}

impl_from_ref! {
    AppState;
    [
        PostEventEmitter => |input: &AppState| input.core.event_emitter.post.clone()
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
    AppState;
    [
        cookie::Key => |input: &AppState| input.session_config.cookie_key.clone(),
        axum_flash::Config => |input: &AppState| input.session_config.flash_config.clone()
    ]
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub core: CoreState,
    pub oauth2: OAuth2Service,
    pub oauth_endpoint: OAuthEndpoint,
    #[cfg(feature = "oidc")]
    pub oidc: Option<OidcService>,
    pub session_config: SessionConfig,
}

// BAD BAD ANTI-PATTERN BAD
impl std::ops::Deref for AppState {
    type Target = CoreState;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}
