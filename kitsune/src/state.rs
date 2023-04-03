use crate::{
    activitypub::Fetcher,
    event::PostEventEmitter,
    service::{
        account::AccountService, attachment::AttachmentService, instance::InstanceService,
        job::JobService, oauth2::Oauth2Service, oidc::OidcService, post::PostService,
        search::SearchService, timeline::TimelineService, url::UrlService, user::UserService,
    },
    webfinger::Webfinger,
};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

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
        JobService => |input: &Zustand| input.service.job.clone(),
        Oauth2Service => |input: &Zustand| input.service.oauth2.clone(),
        Option<OidcService> => |input: &Zustand| input.service.oidc.clone(),
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

/// Service collection
///
/// This contains all the "services" that Kitsune consists of.
/// These are things like the search service, post service, etc.
#[derive(Clone)]
pub struct Service {
    pub account: AccountService,
    pub attachment: AttachmentService,
    pub job: JobService,
    pub oauth2: Oauth2Service,
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
    pub db_conn: DatabaseConnection,
    pub event_emitter: EventEmitter,
    pub fetcher: Fetcher,
    #[cfg(feature = "mastodon-api")]
    pub mastodon_mapper: crate::mapping::MastodonMapper,
    pub service: Service,
    pub webfinger: Webfinger,
}
