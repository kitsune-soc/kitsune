#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

use diesel::Connection;
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing_log::LogTracer;

pub use crate::{
    error::{Error, Result},
    pool::{PgPool, PoolError},
};

mod error;
mod pool;

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
pub async fn connect(conn_str: &str, max_pool_size: usize) -> Result<PgPool> {
    LogTracer::init().ok();

    tokio::task::spawn_blocking({
        let conn_str = conn_str.to_string();

        move || {
            let mut migration_conn =
                AsyncConnectionWrapper::<AsyncPgConnection>::establish(conn_str.as_str())?;

            migration_conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(Error::Migration)?;

            Ok::<_, Error>(())
        }
    })
    .await??;

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(conn_str);
    let pool = Pool::builder(config)
        .max_size(max_pool_size)
        .build()
        .unwrap();

    let mut conn = pool.get().await?;

    kitsune_language::generate_postgres_enum(&mut conn, "language_iso_code").await?;
    kitsune_language::generate_regconfig_function(
        &mut conn,
        "iso_code_to_language",
        "language_iso_code",
    )
    .await?;

    Ok(pool.into())
}
