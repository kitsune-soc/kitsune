pub fn watch(config: &str) -> anyhow::Result<()> {
    let run_cmd = format!("cargo run -p kitsune -- -c {config}");

    crate::util::cargo(["watch", "-x", "check", "-s", &run_cmd])
}
