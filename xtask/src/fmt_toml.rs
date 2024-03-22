use glob::glob;
use std::fs;

fn formatter_settings() -> taplo::formatter::Options {
    taplo::formatter::Options {
        indent_entries: true,
        indent_tables: true,
        reorder_arrays: true,
        reorder_keys: true,
        ..Default::default()
    }
}

pub fn fmt() -> anyhow::Result<()> {
    let mut path_iter = glob("**/*.toml")?;
    while let Some(toml_path) = path_iter.next().transpose()? {
        let toml_data = fs::read_to_string(&toml_path)?;
        let formatted = taplo::formatter::format(&toml_data, formatter_settings());
        fs::write(&toml_path, formatted)?;
    }

    Ok(())
}
