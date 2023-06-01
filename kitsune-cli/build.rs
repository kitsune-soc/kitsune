use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder()
        .all_git()
        .git_sha(true)
        .emit_and_set()
        .unwrap();
}
