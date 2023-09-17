use clap::Parser;
use color_eyre::eyre;
use kitsune_core::config::Configuration;
use std::path::PathBuf;
use tokio::fs;

/// Dedicated Kitsune job runner
#[derive(Parser)]
#[command(about, author, version)]
struct Args {
    /// Path to the configuration
    #[arg(long, short)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let raw_config = fs::read_to_string(args.config).await?;
    let config: Configuration = toml::from_str(&raw_config)?;

    let db_pool = kitsune_db::connect(
        &config.database.url,
        config.database.max_connections as usize,
    )
    .await?;
    let job_queue = kitsune_job_runner::prepare_job_queue(db_pool.clone(), &config.job_queue)?;
    let state = kitsune_core::prepare_state(&config, db_pool, job_queue.clone()).await?;

    kitsune_job_runner::run_dispatcher(job_queue, state, config.job_queue.num_workers.into()).await;

    Ok(())
}
