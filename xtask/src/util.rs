use anyhow::bail;
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

    if !output.status.success() {
        let msg = String::from_utf8(output.stderr)?;
        bail!("Failed to run cargo subcommand: {msg}");
    }

    Ok(())
}
