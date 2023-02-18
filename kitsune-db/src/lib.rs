//!
//! Database crate for Kitsune
//!
//! Contains all the database entities and boilerplate to define all the relationships
//!
//! **Important**: If you ever regenerate the database entities, make sure to fix all the fields that contain enum values.
//! These fields are defined as integers in the database and the CLI will generate them as such.
//!
//! Also, please generate the entities from a Postgres database. Postgres will generate the most accurate entities.
//!

#![forbid(rust_2018_idioms)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, forbidden_lint_groups)]

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};
use tracing_log::LogTracer;

pub mod column;
pub mod custom;
#[allow(missing_docs)]
pub mod entity; // Allow missing docs in this module since its almost fully autogenerated by `SeaORM` CLI
pub mod link;
pub mod r#trait;

/// Connect to a database and run the migrations
///
/// # Errors
///
/// - Connection could not be established
/// - Running the migration failed
pub async fn connect(db_url: &str) -> Result<DatabaseConnection, DbErr> {
    LogTracer::init().ok();

    let conn = Database::connect(db_url).await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
}
