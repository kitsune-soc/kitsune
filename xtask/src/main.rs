#[macro_use]
extern crate tracing;

use std::path::PathBuf;

mod build_scss;

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "build-scss")]
/// Build a directory of SCSS files
struct BuildScss {
    #[argh(option, default = "\"kitsune/assets\".into()")]
    /// path to the directory
    path: PathBuf,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    BuildScss(BuildScss),
}

#[derive(argh::FromArgs)]
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
    }

    Ok(())
}
