use eyre::Result;
use glob::glob;
use grass_compiler::{Options, OutputStyle};
use std::{fs, path::Path};

pub fn compile<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let scss_options = Options::default().style(OutputStyle::Compressed);

    let pattern = format!("{}/*.scss", path.display());
    for file in glob(&pattern)? {
        let mut path = file?;
        tracing::info!("Compiling \"{}\" into CSS", path.display());

        let compiled_css = grass_compiler::from_path(&path, &scss_options)?;
        path.set_extension("css");
        fs::write(path, compiled_css)?;
    }

    Ok(())
}
