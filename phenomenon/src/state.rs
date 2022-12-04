use crate::{config::Configuration, fetcher::Fetcher, webfinger::Webfinger};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;
use std::{ops::Deref, sync::Arc};

/// Clonable wrapper around a Redis connection pool
#[derive(Clone)]
pub struct RedisConnection(Arc<deadpool_redis::Manager>);

impl From<deadpool_redis::Manager> for RedisConnection {
    fn from(manager: deadpool_redis::Manager) -> Self {
        Self(Arc::new(manager))
    }
}

impl Deref for RedisConnection {
    type Target = deadpool_redis::Manager;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
    pub redis_conn: RedisConnection,
    pub webfinger: Webfinger,
}
