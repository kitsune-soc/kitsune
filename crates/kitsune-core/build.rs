use vergen::EmitBuilder;

fn main() {
    println!("cargo:rerun-if-changed=templates");

    EmitBuilder::builder()
        .all_git()
        .git_sha(true)
        .emit_and_set()
        .unwrap();
}
