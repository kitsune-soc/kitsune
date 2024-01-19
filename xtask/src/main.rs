#[macro_use]
extern crate tracing;

use argh::FromArgs;
use std::path::PathBuf;

mod build_scss;
mod util;
mod watch;

#[derive(FromArgs)]
#[argh(subcommand, name = "build-scss")]
/// Build a directory of SCSS files
struct BuildScss {
    #[argh(option)]
    /// path to the directory
    path: PathBuf,
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
    BuildScss(BuildScss),
    Watch(Watch),
}

#[derive(FromArgs)]
/// Kitsune dev taskrunner
struct Command {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let command: Command = argh::from_env();
    match command.subcommand {
        Subcommand::BuildScss(BuildScss { path }) => build_scss::build_scss(path)?,
        Subcommand::Watch(Watch { config, bin }) => watch::watch(&config, &bin)?,
    }

    Ok(())
}
