#[macro_use]
extern crate tracing;

use diesel::Connection;
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use kitsune_config::database::Configuration as DatabaseConfig;
use kitsune_error::{Error, Result};
use tracing_log::LogTracer;

pub type PgPool = Pool<AsyncPgConnection>;

#[doc(hidden)]
pub use {diesel_async, trials};

mod error;
mod pool;
mod tls;

pub mod activity;
pub mod function;
pub mod json;
pub mod lang;
pub mod model;
pub mod post_permission_check;
#[allow(clippy::wildcard_imports)]
pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Connect to the database and run any pending migrations
pub async fn connect(config: &DatabaseConfig) -> Result<PgPool> {
    LogTracer::init().ok();

    blowocking::io({
        let conn_str = config.url.clone();

        move || {
            let mut migration_conn =
                AsyncConnectionWrapper::<AsyncPgConnection>::establish(conn_str.as_str())?;

            migration_conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(Error::msg)?;

            Ok::<_, Error>(())
        }
    })
    .await??;

    let pool_config = if config.use_tls {
        AsyncDieselConnectionManager::new_with_config(config.url.as_str(), self::tls::pool_config())
    } else {
        AsyncDieselConnectionManager::new(config.url.as_str())
    };

    let pool = Pool::builder()
        .max_size(config.max_connections)
        .build(pool_config)
        .await
        .unwrap();

    {
        let mut conn = pool.get().await?;

        kitsune_language::generate_postgres_enum(&mut conn, "language_iso_code").await?;
        kitsune_language::generate_regconfig_function(
            &mut conn,
            "iso_code_to_language",
            "language_iso_code",
        )
        .await?;
    }

    Ok(pool)
}
