use self::{config::Configuration, role::RoleSubcommand};
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use kitsune_config::database::Configuration as DatabaseConfig;

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
    color_eyre::install()?;
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env()?;
    let db_conn = kitsune_db::connect(&DatabaseConfig {
        url: config.database_url.into(),
        max_connections: 1,
        use_tls: config.database_use_tls,
    })
    .await?;

    let cmd = App::parse();

    let mut db_conn = db_conn.get().await?;
    match cmd.subcommand {
        AppSubcommand::Role(cmd) => self::role::handle(cmd, &mut db_conn).await?,
    }

    Ok(())
}
