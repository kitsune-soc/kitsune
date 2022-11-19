use crate::error::Result;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub mod entity;

pub async fn connect(db_url: &str) -> Result<DatabaseConnection> {
    let conn = Database::connect(db_url).await?;
    Migrator::up(&conn, None).await?;
    Ok(conn)
}
