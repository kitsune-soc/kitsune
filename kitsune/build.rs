use std::{io, process::Command};

/// Run an `xtask` subcommand
fn xtask(args: &[&str]) -> io::Result<()> {
    let output = Command::new(env!("CARGO"))
        .args(["run", "--manifest-path", "../xtask/Cargo.toml", "--"])
        .args(args)
        .output()?;

    output.status.success().then_some(()).ok_or_else(|| {
        let stderr = String::from_utf8(output.stderr).unwrap();
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to run xtask: {stderr}"),
        )
    })
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=templates");

    xtask(&["build-scss", "--path", "./assets"])?;

    Ok(())
}
