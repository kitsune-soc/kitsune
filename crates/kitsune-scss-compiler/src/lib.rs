use anyhow::Result;
use glob::glob;
use rsass::output::{Format, Style};
use std::{fs, path::Path};

pub fn compile<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let scss_format = Format {
        style: Style::Compressed,
        ..Default::default()
    };

    let pattern = format!("{}/*.scss", path.display());
    for file in glob(&pattern)? {
        let mut path = file?;
        tracing::info!("Compiling \"{}\" into CSS", path.display());

        let compiled_css = rsass::compile_scss_path(&path, scss_format)?;
        path.set_extension("css");
        fs::write(path, compiled_css)?;
    }

    Ok(())
}
