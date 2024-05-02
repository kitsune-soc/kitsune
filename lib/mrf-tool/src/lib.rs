use self::args::{ManifestSubcommand, ModuleSubcommand, ToolArgs, ToolSubcommand};
use clap::Parser;
use color_eyre::{eyre::bail, Result};
use std::{ffi::OsString, io::Write, path::Path};

pub use self::fs::{Filesystem, NativeFs};

mod args;
mod fs;

pub fn read_manifest<W>(sink: &mut W, module: &[u8]) -> Result<()>
where
    W: Write,
{
    let Some((manifest, _section_range)) = mrf_manifest::decode(module)? else {
        bail!("missing manifest in module");
    };

    let prettified = serde_json::to_string_pretty(&manifest)?;
    writeln!(sink, "{prettified}")?;

    Ok(())
}

pub fn remove_manifest<F>(fs: &mut F, module_path: &Path, output_path: &Path) -> Result<()>
where
    F: Filesystem,
{
    let module = fs.read(module_path)?;
    let Some((_manifest, section_range)) = mrf_manifest::decode(&module)? else {
        bail!("missing manifest in module");
    };

    let mut module_file = fs.create_or_truncate(output_path)?;
    module_file.write_all(&module[..section_range.start])?;
    module_file.write_all(&module[section_range.end..])?;

    Ok(())
}

pub fn write_manifest<F>(fs: &mut F, manifest: &[u8], module_path: &Path) -> Result<()>
where
    F: Filesystem,
{
    // Parse the manifest and re-encode it in canonical JSON
    let parsed_manifest = serde_json::from_slice(manifest)?;
    let custom_section = mrf_manifest::encode(&parsed_manifest)?;

    let mut file = fs.open_append(module_path)?;
    file.write_all(&custom_section)?;

    Ok(())
}

pub fn handle<F, W, I>(fs: &mut F, sink: &mut W, input: I) -> Result<()>
where
    F: Filesystem,
    W: Write,
    I: IntoIterator,
    <I as IntoIterator>::Item: Into<OsString> + Clone,
{
    let args = ToolArgs::try_parse_from(input)?;
    match args.command {
        ToolSubcommand::Manifest(ManifestSubcommand::Add(args)) => {
            let manifest = fs.read(&args.manifest_path)?;

            // Only copy if the paths are distinct
            if args.module_path != args.output {
                fs.copy(&args.module_path, &args.output)?;
            }

            write_manifest(fs, &manifest, &args.output)?;
        }
        ToolSubcommand::Manifest(ManifestSubcommand::Read(args)) => {
            let data = fs.read(&args.module_path)?;
            read_manifest(sink, &data)?;
        }
        ToolSubcommand::Manifest(ManifestSubcommand::Remove(args)) => {
            remove_manifest(fs, &args.module_path, &args.output)?;
        }
        ToolSubcommand::Module(ModuleSubcommand::Validate(args)) => {
            let data = fs.read(&args.module_path)?;
            wasmparser::validate(&data)?;
        }
    }

    Ok(())
}
