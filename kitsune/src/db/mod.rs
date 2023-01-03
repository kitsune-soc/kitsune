use crate::error::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{prelude::*, Database};

pub mod model;

#[derive(Copy, Clone, Debug, DeriveColumn, EnumIter)]
pub enum InboxUrlQuery {
    InboxUrl,
}

#[derive(Copy, Clone, Debug, DeriveColumn, EnumIter)]
pub enum UrlQuery {
    Url,
}

/// Connect to a database and run the migrations
///
/// # Errors
///
/// - Connection could not be established
/// - Running the migration failed
pub async fn connect(db_url: &str) -> Result<DatabaseConnection> {
    let conn = Database::connect(db_url).await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
}
