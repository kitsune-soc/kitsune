use std::env;
use vergen::EmitBuilder;

const PRIMARY_PACKAGE_KEY: &str = "CARGO_PRIMARY_PACKAGE";
const PRIMARY_PACKAGE_FALLBACK_VALUE: &str = "kitsune";

fn main() {
    if env::var(PRIMARY_PACKAGE_KEY).is_err() {
        println!("cargo:rustc-env={PRIMARY_PACKAGE_KEY}={PRIMARY_PACKAGE_FALLBACK_VALUE}");
    }

    EmitBuilder::builder()
        .all_git()
        .git_sha(true)
        .emit_and_set()
        .unwrap();
}
