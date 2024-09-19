use crate::util::cargo;

pub fn clean() -> eyre::Result<()> {
    cargo(["clean"])?;
    cargo(["clean", "--target-dir", "target-analyzer"])?;

    Ok(())
}
