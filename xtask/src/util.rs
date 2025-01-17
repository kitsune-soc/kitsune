use eyre::ensure;
use std::{env, ffi::OsStr, process::Command};

pub fn cargo<I>(params: I) -> eyre::Result<()>
where
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    let cargo = env::var("CARGO").unwrap();
    let status = Command::new(cargo).args(params).status()?;

    ensure!(status.success(), "Failed to run cargo subcommand");

    Ok(())
}
