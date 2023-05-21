use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations_async::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing_log::LogTracer;

pub use crate::error::{Error, Result};

mod error;
mod macros;

pub mod function;
pub mod model;
pub mod post_permission_check;
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
pub type PgPool = Pool<AsyncPgConnection>;

/// Connect to the database and run any pending migrations
pub async fn connect(conn_str: &str, max_pool_size: usize) -> Result<PgPool> {
    LogTracer::init().ok();

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(conn_str);
    let pool = Pool::builder(config)
        .max_size(max_pool_size)
        .build()
        .unwrap();

    let mut conn = pool.get().await?;
    conn.run_pending_migrations(MIGRATIONS)
        .await
        .map_err(Error::Migration)?;

    Ok(pool)
}
