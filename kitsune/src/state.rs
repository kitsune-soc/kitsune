use crate::oauth2::OAuth2Service;
use axum_extra::extract::cookie;
use kitsune_config::language_detection::Configuration as LanguageDetectionConfig;
use kitsune_core::traits::Fetcher;
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
use kitsune_wasm_mrf::MrfService;
use std::ops::Deref;
use triomphe::Arc;

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
        MastodonMapper => |input: &Zustand| input.mastodon_mapper.clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        AccountService => |input: &Zustand| input.service.account.clone(),
        AttachmentService => |input: &Zustand| input.service.attachment.clone(),
        CustomEmojiService => |input: &Zustand| input.service.custom_emoji.clone(),
        FederationFilter => |input: &Zustand| input.federation_filter.clone(),
        JobService => |input: &Zustand| input.service.job.clone(),
        MailingService => |input: &Zustand| input.service.mailing.clone(),
        MrfService => |input: &Zustand| input.service.mrf.clone(),
        NotificationService => |input: &Zustand| input.service.notification.clone(),
        PostService => |input: &Zustand| input.service.post.clone(),
        SearchService => |input: &Zustand| input.service.search.clone(),
        InstanceService => |input: &Zustand| input.service.instance.clone(),
        TimelineService => |input: &Zustand| input.service.timeline.clone(),
        UrlService => |input: &Zustand| input.service.url.clone(),
        UserService => |input: &Zustand| input.service.user.clone()
    ]
}

#[derive(Clone)]
pub struct SessionConfig {
    pub cookie_key: cookie::Key,
}

impl SessionConfig {
    /// Randomly generates the keys for the cookie jars
    #[must_use]
    pub fn generate() -> Self {
        let cookie_key = cookie::Key::generate();

        Self { cookie_key }
    }
}

impl_from_ref! {
    Zustand;
    [
        cookie::Key => |input: &Zustand| input.session_config.cookie_key.clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        OAuth2Service => |input: &Zustand| input.oauth2.clone(),
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
    pub mrf: MrfService,
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
    pub federation_filter: FederationFilter,
    pub fetcher: Arc<dyn Fetcher>,
    pub language_detection_config: LanguageDetectionConfig,
    #[cfg(feature = "mastodon-api")]
    pub mastodon_mapper: MastodonMapper,
    pub oauth2: OAuth2Service,
    #[cfg(feature = "oidc")]
    pub oidc: Option<OidcService>,
    pub service: Service,
    pub session_config: SessionConfig,
}

impl ZustandInner {
    #[must_use]
    pub fn ap_state(&self) -> kitsune_activitypub::mapping::State<'_> {
        kitsune_activitypub::mapping::State::builder()
            .db_pool(&self.db_pool)
            .service(
                kitsune_activitypub::mapping::Service::builder()
                    .attachment(&self.service.attachment)
                    .url(&self.service.url)
                    .build(),
            )
            .build()
    }
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
