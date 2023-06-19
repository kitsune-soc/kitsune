#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, forbidden_lint_groups)]

use self::{config::Configuration, role::RoleSubcommand};
use clap::{Parser, Subcommand};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use kitsune_db::{
    model::job::JobState,
    schema::jobs::dsl::{jobs, state},
};
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
#[command(about, author, version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("VERGEN_GIT_SHA")))]
struct App {
    #[clap(subcommand)]
    subcommand: AppSubcommand,
}

async fn clear_completed_jobs(db_conn: &mut AsyncPgConnection) -> Result<()> {
    let delete_result = diesel::delete(jobs.filter(state.eq(JobState::Succeeded)))
        .execute(db_conn)
        .await?;

    println!("Deleted {delete_result} succeeded jobs from the database");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env()?;
    let db_conn = kitsune_db::connect(&config.database_url, 1).await?;
    let mut db_conn = db_conn.get().await?;
    let cmd = App::parse();

    match cmd.subcommand {
        AppSubcommand::ClearSucceededJobs => clear_completed_jobs(&mut db_conn).await?,
        AppSubcommand::Role(cmd) => self::role::handle(cmd, &mut db_conn).await?,
    }

    Ok(())
}
