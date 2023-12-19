pub fn watch(config: &str, bin: &str) -> anyhow::Result<()> {
    let run_cmd = format!("cargo run -p {bin} -- -c {config}");

    crate::util::cargo(["watch", "-x", "check", "-s", &run_cmd])
}
