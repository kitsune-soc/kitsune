#[macro_use]
extern crate tracing;

use argh::FromArgs;

mod clean;
mod download_ap_fixture;
mod util;
mod watch;

#[derive(FromArgs)]
#[argh(subcommand, name = "clean")]
/// Clean all target directories
struct Clean {}

#[derive(FromArgs)]
#[argh(subcommand, name = "download-ap-fixture")]
/// Download ActivityPub fixtures
struct DownloadApFixture {
    #[argh(positional)]
    url: String,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "watch")]
/// Watch for source changes and automatically check the code and run the server
struct Watch {
    #[argh(option, short = 'c', default = "\"config.toml\".into()")]
    /// path to the configuration file
    config: String,

    #[argh(option, short = 'p', default = "\"kitsune\".into()")]
    /// name of the binary in the workspace
    bin: String,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Clean(Clean),
    DownloadApFixture(DownloadApFixture),
    Watch(Watch),
}

#[derive(FromArgs)]
/// Kitsune dev taskrunner
struct Command {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let command: Command = argh::from_env();
    match command.subcommand {
        Subcommand::Clean(..) => clean::clean()?,
        Subcommand::DownloadApFixture(DownloadApFixture { url }) => {
            download_ap_fixture::download(&url)?;
        }
        Subcommand::Watch(Watch { config, bin }) => watch::watch(&config, &bin)?,
    }

    Ok(())
}
