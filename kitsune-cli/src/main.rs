#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, forbidden_lint_groups)]

use self::{config::Configuration, role::RoleSubcommand};
use clap::{Parser, Subcommand};
use std::error::Error;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

mod config;
mod role;

#[derive(Subcommand)]
enum AppSubcommand {
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env()?;
    let db_conn = kitsune_db::connect(&config.database_url, 1).await?;
    let mut db_conn = db_conn.get().await?;
    let cmd = App::parse();

    match cmd.subcommand {
        AppSubcommand::Role(cmd) => self::role::handle(cmd, &mut db_conn).await?,
    }

    Ok(())
}
