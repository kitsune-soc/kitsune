use camino::Utf8Path;
use fs_extra::dir::{self, CopyOptions};
use std::{env, error::Error, io, process::Command};

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

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=templates");

    let assets_path = Utf8Path::new("./assets").canonicalize_utf8()?;
    let prepared_assets_path = Utf8Path::new("./assets-dist");

    // Only clean the `assets-dist` directory on non-debug builds
    if !cfg!(debug_assertions) {
        dir::remove(prepared_assets_path)?;
    }

    let copy_options = CopyOptions {
        overwrite: true,
        content_only: true,
        ..CopyOptions::default()
    };
    dir::copy(assets_path, prepared_assets_path, &copy_options)?;

    xtask(&["build-scss", "--path", prepared_assets_path.as_str()])?;

    Ok(())
}
