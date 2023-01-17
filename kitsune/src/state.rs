use crate::{
    activitypub::Fetcher, config::Configuration, search::GrpcSearchService, webfinger::Webfinger,
};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

/// Application state
///
/// Called it "Zustand" to avoid a name collission with `axum::extract::State`.
/// "Zustand" is just the german word for state.
#[derive(Clone, FromRef)]
pub struct Zustand {
    pub config: Configuration,
    pub db_conn: DatabaseConnection,
    pub fetcher: Fetcher,
    pub search_service: GrpcSearchService,
    pub webfinger: Webfinger,
}
