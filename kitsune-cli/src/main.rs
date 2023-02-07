use self::{config::Configuration, role::RoleSubcommand};
use clap::{Parser, Subcommand};
use kitsune_db::{
    custom::JobState,
    entity::{jobs, prelude::Jobs},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::error::Error;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

mod config;
mod role;

#[derive(Subcommand)]
enum AppSubcommand {
    /// Clear succeeded jobs from database
    ///
    /// Succeeded jobs are kept in the database so administrators can aggregate some nice statistics.  
    /// However, they can fill up your database and aren't essential to anything.
    ClearSucceededJobs,

    /// Manage roles for local users
    #[clap(subcommand)]
    Role(RoleSubcommand),
}

/// CLI for the Kitsune social media server
#[derive(Parser)]
#[command(about, author, version)]
struct App {
    #[clap(subcommand)]
    subcommand: AppSubcommand,
}

async fn clear_completed_jobs(db_conn: DatabaseConnection) -> Result<()> {
    let delete_result = Jobs::delete_many()
        .filter(jobs::Column::State.eq(JobState::Succeeded))
        .exec(&db_conn)
        .await?;

    println!(
        "Deleted {} succeeded jobs from the database",
        delete_result.rows_affected
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env()?;
    let db_conn = kitsune_db::connect(&config.database_url).await?;
    let cmd = App::parse();

    match cmd.subcommand {
        AppSubcommand::ClearSucceededJobs => clear_completed_jobs(db_conn).await?,
        AppSubcommand::Role(cmd) => self::role::handle(cmd, db_conn).await?,
    }

    Ok(())
}
