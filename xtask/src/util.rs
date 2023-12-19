use anyhow::ensure;
use std::{env, ffi::OsStr, io, process::Command};

pub fn cargo<I>(params: I) -> anyhow::Result<()>
where
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    let cargo = env::var("CARGO").unwrap();
    let output = Command::new(cargo)
        .args(params)
        .stderr(io::stderr())
        .stdout(io::stdout())
        .output()?;

    ensure!(output.status.success(), "Failed to run cargo subcommand");

    Ok(())
}
