#[macro_use]
extern crate tracing;

use clap::Parser;

mod clean;
mod download_ap_fixture;
mod util;
mod watch;

#[derive(clap::Args)]
/// Clean all target directories
struct Clean {}

#[derive(clap::Args)]
/// Download ActivityPub fixtures
struct DownloadApFixture {
    url: String,
}

#[derive(clap::Args)]
/// Watch for source changes and automatically check the code and run the server
struct Watch {
    #[clap(short = 'c', default_value = "config.toml")]
    /// path to the configuration file
    config: String,

    #[clap(short = 'p', default_value = "kitsune")]
    /// name of the binary in the workspace
    bin: String,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Clean(Clean),
    DownloadApFixture(DownloadApFixture),
    Watch(Watch),
}

#[derive(Parser)]
/// Kitsune dev taskrunner
struct Command {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let command = Command::parse();
    match command.subcommand {
        Subcommand::Clean(..) => clean::clean()?,
        Subcommand::DownloadApFixture(DownloadApFixture { url }) => {
            download_ap_fixture::download(&url)?;
        }
        Subcommand::Watch(Watch { config, bin }) => watch::watch(&config, &bin)?,
    }

    Ok(())
}
