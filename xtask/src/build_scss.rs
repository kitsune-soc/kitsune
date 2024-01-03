use std::path::PathBuf;

#[inline]
pub fn build_scss(path: PathBuf) -> anyhow::Result<()> {
    info!("Building backend SCSS..");
    kitsune_scss_compiler::compile(path)?;

    Ok(())
}
