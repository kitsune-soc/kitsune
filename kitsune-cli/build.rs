use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder()
        .all_git()
        .git_sha(true)
        .emit_and_set()
        .unwrap_or_else(|_| println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown"));
}
