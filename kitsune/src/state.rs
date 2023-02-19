use crate::{
    activitypub::Fetcher,
    config::Configuration,
    service::{
        account::AccountService, post::PostService, search::SearchService, user::UserService,
    },
    webfinger::Webfinger,
};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// Service collection
///
/// This contains all the "services" that Kitsune consists of.
/// These are things like the search service, post service, etc.
#[derive(Clone)]
pub struct Service {
    pub account: AccountService,
    pub post: PostService,
    pub search: Arc<dyn SearchService + Send + Sync>,
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
