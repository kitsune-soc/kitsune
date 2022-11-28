use crate::{config::Configuration, fetcher::Fetcher, webfinger::Webfinger};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

#[derive(Clone, FromRef)]
pub struct State {
    pub db_conn: DatabaseConnection,
    pub config: Configuration,
    pub fetcher: Fetcher,
    pub webfinger: Webfinger,
}
