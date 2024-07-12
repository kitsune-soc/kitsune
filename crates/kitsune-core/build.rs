use std::env;
use vergen_gitcl::{Emitter, GitclBuilder};

const PRIMARY_PACKAGE_KEY: &str = "CARGO_PRIMARY_PACKAGE";
const PRIMARY_PACKAGE_FALLBACK_VALUE: &str = "kitsune";

fn main() {
    if env::var(PRIMARY_PACKAGE_KEY).is_err() {
        println!("cargo:rustc-env={PRIMARY_PACKAGE_KEY}={PRIMARY_PACKAGE_FALLBACK_VALUE}");
    }

    let gitcl = GitclBuilder::default().sha(true).build().unwrap();

    Emitter::new()
        .add_instructions(&gitcl)
        .unwrap()
        .emit()
        .unwrap();
}
