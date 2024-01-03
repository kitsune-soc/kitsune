use camino::Utf8Path;
use fs_extra::dir::{self, CopyOptions};
use std::error::Error;

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

    kitsune_scss_compiler::compile(prepared_assets_path)?;

    Ok(())
}
