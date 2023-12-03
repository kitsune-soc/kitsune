use crate::oauth2::{OAuth2Service, OAuthEndpoint};
use axum_extra::extract::cookie;
use kitsune_core::event::PostEventEmitter;
use kitsune_db::PgPool;
use kitsune_email::MailingService;
use kitsune_embed::Client as EmbedClient;
use kitsune_federation_filter::FederationFilter;
use kitsune_service::{
    account::AccountService, attachment::AttachmentService, captcha::CaptchaService,
    custom_emoji::CustomEmojiService, instance::InstanceService, job::JobService,
    notification::NotificationService, post::PostService, search::SearchService,
    timeline::TimelineService, user::UserService,
};
use kitsune_url::UrlService;
use std::{ops::Deref, sync::Arc};

#[cfg(feature = "mastodon-api")]
use kitsune_mastodon::MastodonMapper;

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
        PgPool => |input: &Zustand| input.db_pool.clone()
    ]
}

#[cfg(feature = "mastodon-api")]
impl_from_ref! {
    Zustand;
    [
        MastodonMapper => |input: &Zustand| input.mastodon_mapper().clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        AccountService => |input: &Zustand| input.service.account.clone(),
        AttachmentService => |input: &Zustand| input.service.attachment.clone(),
        CustomEmojiService => |input: &Zustand| input.service.custom_emoji.clone(),
        FederationFilter => |input: &Zustand| input.service.federation_filter.clone(),
        JobService => |input: &Zustand| input.service.job.clone(),
        MailingService => |input: &Zustand| input.service.mailing.clone(),
        NotificationService => |input: &Zustand| input.service.notification.clone(),
        PostService => |input: &Zustand| input.service.post.clone(),
        SearchService => |input: &Zustand| input.service.search.clone(),
        InstanceService => |input: &Zustand| input.service.instance.clone(),
        TimelineService => |input: &Zustand| input.service.timeline.clone(),
        UrlService => |input: &Zustand| input.service.url.clone(),
        UserService => |input: &Zustand| input.service.user.clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        PostEventEmitter => |input: &Zustand| input.event_emitter().post.clone()
    ]
}

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

/// Emitter collection
///
/// This contains all the "emitters" that can emit events inside of Kitsune.
/// Something like "a post has been created" or "an account has been followed".
pub struct EventEmitter {
    pub post: PostEventEmitter,
}

/// Service collection
///
/// This contains all the "services" that Kitsune consists of.
/// These are things like the search service, post service, etc.
pub struct Service {
    pub account: AccountService,
    pub attachment: AttachmentService,
    pub captcha: CaptchaService,
    pub custom_emoji: CustomEmojiService,
    pub job: JobService,
    pub mailing: MailingService,
    pub notification: NotificationService,
    pub post: PostService,
    pub instance: InstanceService,
    pub search: SearchService,
    pub timeline: TimelineService,
    pub url: UrlService,
    pub user: UserService,
}

pub struct ZustandInner {
    pub db_pool: PgPool,
    pub embed_client: Option<EmbedClient>,
    pub event_emitter: EventEmitter,
    pub federation_filter: FederationFilter,
    #[cfg(feature = "mastodon-api")]
    pub mastodon_mapper: MastodonMapper,
    pub oauth2: OAuth2Service,
    pub oauth_endpoint: OAuthEndpoint,
    #[cfg(feature = "oidc")]
    pub oidc: Option<OidcService>,
    pub service: Service,
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
