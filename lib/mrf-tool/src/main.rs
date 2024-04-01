use self::args::{ManifestSubcommand, ModuleSubcommand, ToolArgs, ToolSubcommand};
use clap::Parser;
use color_eyre::{eyre::bail, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

mod args;

fn read_manifest(module: &[u8]) -> Result<()> {
    let Some((manifest, _section_range)) = mrf_manifest::decode(module)? else {
        bail!("missing manifest in module");
    };

    let prettified = serde_json::to_string_pretty(&manifest)?;
    println!("{prettified}");

    Ok(())
}

fn remove_manifest(module_path: &Path, output_path: &Path) -> Result<()> {
    let module = fs::read(module_path)?;
    let Some((_manifest, section_range)) = mrf_manifest::decode(&module)? else {
        bail!("missing manifest in module");
    };

    let mut module_file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output_path)?;

    module_file.write_all(&module[..section_range.start])?;
    module_file.write_all(&module[section_range.end..])?;

    Ok(())
}

fn write_manifest(manifest: &[u8], module_path: &Path) -> Result<()> {
    // Parse the manifest and re-encode it in canonical JSON
    let parsed_manifest = serde_json::from_slice(manifest)?;
    let custom_section = mrf_manifest::encode(&parsed_manifest)?;

    let mut file = File::options().append(true).open(module_path)?;
    file.write_all(&custom_section)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = ToolArgs::parse();
    match args.command {
        ToolSubcommand::Manifest(ManifestSubcommand::Add(args)) => {
            let manifest = fs::read(args.manifest_path)?;

            // Only copy if the paths are distinct
            if args.module_path != args.output {
                fs::copy(&args.module_path, &args.output)?;
            }

            write_manifest(&manifest, &args.output)?;
        }
        ToolSubcommand::Manifest(ManifestSubcommand::Read(args)) => {
            let data = fs::read(args.module_path)?;
            read_manifest(&data)?;
        }
        ToolSubcommand::Manifest(ManifestSubcommand::Remove(args)) => {
            remove_manifest(&args.module_path, &args.output)?;
        }
        ToolSubcommand::Module(ModuleSubcommand::Validate(args)) => {
            let data = fs::read(args.module_path)?;
            wasmparser::validate(&data)?;
        }
    }

    Ok(())
}
