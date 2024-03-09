use crate::util::cargo;

pub fn clean() -> anyhow::Result<()> {
    cargo(["clean"])?;
    cargo(["clean", "--target-dir", "target-analyzer"])?;

    Ok(())
}
