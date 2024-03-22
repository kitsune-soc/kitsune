use glob::glob;
use std::fs;

fn formatter_settings() -> taplo::formatter::Options {
    taplo::formatter::Options {
        indent_string: " ".repeat(4),
        reorder_arrays: true,
        ..Default::default()
    }
}

pub fn fmt() -> anyhow::Result<()> {
    let mut path_iter = glob("**/*.toml")?;
    while let Some(toml_path) = path_iter.next().transpose()? {
        info!(path = %toml_path.display(), "formatting TOML file");
        let toml_data = fs::read_to_string(&toml_path)?;
        let formatted = taplo::formatter::format(&toml_data, formatter_settings());
        fs::write(&toml_path, formatted)?;
    }

    Ok(())
}
