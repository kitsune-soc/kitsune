use crate::{config::Configuration, fetcher::Fetcher, webfinger::Webfinger};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct State {
    pub db_conn: DatabaseConnection,
    pub config: Configuration,
    pub fetcher: Fetcher,
    pub webfinger: Webfinger,
}
