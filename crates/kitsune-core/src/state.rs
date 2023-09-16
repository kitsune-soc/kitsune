use crate::{
    activitypub::Fetcher,
    event::PostEventEmitter,
    service::{
        account::AccountService, attachment::AttachmentService, captcha::CaptchaService,
        federation_filter::FederationFilterService, instance::InstanceService, job::JobService,
        mailing::MailingService, notification::NotificationService, post::PostService,
        timeline::TimelineService, url::UrlService, user::UserService,
    },
    webfinger::Webfinger,
};
use kitsune_db::PgPool;
use kitsune_embed::Client as EmbedClient;
use kitsune_search::SearchService;

/// Emitter collection
///
/// This contains all the "emitters" that can emit events inside of Kitsune.
/// Something like "a post has been created" or "an account has been followed".
#[derive(Clone)]
pub struct EventEmitter {
    pub post: PostEventEmitter,
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
    pub notification: NotificationService,
    pub post: PostService,
    pub instance: InstanceService,
    pub search: SearchService,
    pub timeline: TimelineService,
    pub url: UrlService,
    pub user: UserService,
}

/// Core application state
#[derive(Clone)]
pub struct State {
    pub db_pool: PgPool,
    pub embed_client: Option<EmbedClient>,
    pub event_emitter: EventEmitter,
    pub fetcher: Fetcher,
    #[cfg(feature = "mastodon-api")]
    pub mastodon_mapper: crate::mapping::MastodonMapper,
    pub service: Service,
    pub webfinger: Webfinger,
}
