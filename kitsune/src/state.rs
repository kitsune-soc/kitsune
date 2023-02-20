use crate::{
    activitypub::Fetcher,
    config::Configuration,
    service::{
        account::AccountService, oauth2::Oauth2Service, post::PostService,
        search::ArcSearchService, timeline::TimelineService, user::UserService,
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
        Oauth2Service => |input: &Zustand| input.service.oauth2.clone(),
        PostService => |input: &Zustand| input.service.post.clone(),
        ArcSearchService => |input: &Zustand| input.service.search.clone(),
        TimelineService => |input: &Zustand| input.service.timeline.clone()
    ]
}

/// Service collection
///
/// This contains all the "services" that Kitsune consists of.
/// These are things like the search service, post service, etc.
#[derive(Clone)]
pub struct Service {
    pub account: AccountService,
    pub oauth2: Oauth2Service,
    pub post: PostService,
    pub search: ArcSearchService,
    pub timeline: TimelineService,
    pub user: UserService,
}

/// Application state
///
/// Called it "Zustand" to avoid a name collission with `axum::extract::State`.
/// "Zustand" is just the german word for state.
#[derive(Clone, FromRef)]
pub struct Zustand {
    pub config: Configuration,
    pub db_conn: DatabaseConnection,
    pub fetcher: Fetcher,
    pub service: Service,
    pub webfinger: Webfinger,
}
