use crate::{
    activitypub::Fetcher,
    event::PostEventEmitter,
    service::{
        account::AccountService,
        attachment::AttachmentService,
        captcha::CaptchaService,
        federation_filter::FederationFilterService,
        instance::InstanceService,
        job::JobService,
        mailing::MailingService,
        oauth2::{OAuth2Service, OAuthEndpoint},
        post::PostService,
        timeline::TimelineService,
        url::UrlService,
        user::UserService,
    },
    webfinger::Webfinger,
};
use axum::extract::FromRef;
use axum_extra::extract::cookie;
use kitsune_db::PgPool;
use kitsune_embed::Client as EmbedClient;
use kitsune_search::SearchService;

#[cfg(feature = "oidc")]
use crate::service::oidc::OidcService;

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
        AccountService => |input: &Zustand| input.service.account.clone(),
        AttachmentService => |input: &Zustand| input.service.attachment.clone(),
        FederationFilterService => |input: &Zustand| input.service.federation_filter.clone(),
        JobService => |input: &Zustand| input.service.job.clone(),
        OAuth2Service => |input: &Zustand| input.service.oauth2.clone(),
        PostService => |input: &Zustand| input.service.post.clone(),
        SearchService => |input: &Zustand| input.service.search.clone(),
        InstanceService => |input: &Zustand| input.service.instance.clone(),
        TimelineService => |input: &Zustand| input.service.timeline.clone(),
        UrlService => |input: &Zustand| input.service.url.clone(),
        UserService => |input: &Zustand| input.service.user.clone()
    ]
}

#[cfg(feature = "oidc")]
impl_from_ref! {
    Zustand;
    [
        Option<OidcService> => |input: &Zustand| input.service.oidc.clone()
    ]
}

impl_from_ref! {
    Zustand;
    [
        PostEventEmitter => |input: &Zustand| input.event_emitter.post.clone()
    ]
}

/// Emitter collection
///
/// This contains all the "emitters" that can emit events inside of Kitsune.
/// Something like "a post has been created" or "an account has been followed".
#[derive(Clone)]
pub struct EventEmitter {
    pub post: PostEventEmitter,
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

/// Service collection
///
/// This contains all the "services" that Kitsune consists of.
/// These are things like the search service, post service, etc.
#[derive(Clone)]
pub struct Service {
    pub account: AccountService,
    pub attachment: AttachmentService,
    pub captcha: CaptchaService,
    pub federation_filter: FederationFilterService,
    pub job: JobService,
    pub mailing: MailingService,
    pub oauth2: OAuth2Service,
    #[cfg(feature = "oidc")]
    pub oidc: Option<OidcService>,
    pub post: PostService,
    pub instance: InstanceService,
    pub search: SearchService,
    pub timeline: TimelineService,
    pub url: UrlService,
    pub user: UserService,
}

/// Application state
///
/// Called it "Zustand" to avoid a name collission with `axum::extract::State`.
/// "Zustand" is just the german word for state.
#[derive(Clone, FromRef)]
pub struct Zustand {
    pub db_conn: PgPool,
    pub embed_client: Option<EmbedClient>,
    pub event_emitter: EventEmitter,
    pub fetcher: Fetcher,
    #[cfg(feature = "mastodon-api")]
    pub mastodon_mapper: crate::mapping::MastodonMapper,
    pub oauth_endpoint: OAuthEndpoint,
    pub service: Service,
    pub session_config: SessionConfig,
    pub webfinger: Webfinger,
}
