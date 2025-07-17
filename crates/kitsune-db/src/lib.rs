#[macro_use]
extern crate tracing;

use diesel::Connection;
use diesel_async::{
    AsyncPgConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use kitsune_config::database::Configuration as DatabaseConfig;
use kitsune_error::{Error, Result};

pub type PgPool = Pool<AsyncPgConnection>;

#[doc(hidden)]
pub use {diesel_async, trials};

mod error;
mod pool;
mod tls;

pub mod changeset;
pub mod function;
pub mod insert;
pub mod json;
pub mod lang;
pub mod model;
pub mod post_permission_check;
#[allow(clippy::wildcard_imports)]
pub mod schema;
pub mod types;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Connect to the database and run any pending migrations
pub async fn connect(config: &DatabaseConfig) -> Result<PgPool> {
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
        .await?;

    {
        let mut conn = pool.get().await?;

        kitsune_language::generate_postgres_enum(&mut conn).await?;
        kitsune_language::generate_regconfig_function(&mut conn).await?;
    }

    Ok(pool)
}
