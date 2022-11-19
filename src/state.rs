use sea_orm::DatabaseConnection;

use crate::{config::Configuration, fetcher::Fetcher};

#[derive(Clone)]
pub struct State {
    pub db_conn: DatabaseConnection,
    pub config: Configuration,
    pub fetcher: Fetcher,
}
