use clap::{Args, Parser, Subcommand};
use miette::{bail, IntoDiagnostic, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Args)]
struct AddManifest {
    /// Path to the manifest
    manifest_path: PathBuf,

    /// Path to the WASM module
    module_path: PathBuf,

    /// Path to where the modified WASM module should be written
    #[arg(long, short)]
    output: PathBuf,
}

#[derive(Args)]
struct ReadManifest {
    /// Path to the WASM module
    module_path: PathBuf,
}

#[derive(Args)]
struct RemoveManifest {
    /// Path to the WASM module
    module_path: PathBuf,

    /// Path to where the modified WASM module should be written
    #[arg(long, short)]
    output: PathBuf,
}

#[derive(Args)]
struct ValidateModule {
    /// Path to the WASM module
    module_path: PathBuf,
}

#[derive(Subcommand)]
enum ToolSubcommand {
    /// Add a manifest to a WASM component
    ///
    /// Note: We don't validate whether the WASM component already contains a manifest section.
    /// We simply append a new section.
    AddManifest(AddManifest),

    /// Read the manifest from a WASM component
    ReadManifest(ReadManifest),

    /// Remove the manifest from a WASM component
    RemoveManifest(RemoveManifest),

    /// Validate a WASM module
    ValidateModule(ValidateModule),
}

#[derive(Parser)]
#[command(about, version)]
pub struct ToolArgs {
    #[clap(subcommand)]
    command: ToolSubcommand,
}

fn read_manifest(module: &[u8]) -> Result<()> {
    let Some((manifest, _section_range)) = mrf_manifest::parse(module)? else {
        bail!("missing manifest in module");
    };

    let prettified = serde_json::to_string_pretty(&manifest).into_diagnostic()?;
    println!("{prettified}");

    Ok(())
}

fn remove_manifest(module_path: &Path, output_path: &Path) -> Result<()> {
    let module = fs::read(module_path).into_diagnostic()?;
    let Some((_manifest, section_range)) = mrf_manifest::parse(&module)? else {
        bail!("missing manifest in module");
    };

    let mut module_file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output_path)
        .into_diagnostic()?;

    module_file
        .write_all(&module[..section_range.start])
        .into_diagnostic()?;
    module_file
        .write_all(&module[section_range.end..])
        .into_diagnostic()?;

    Ok(())
}

fn write_manifest(manifest: &[u8], module_path: &Path) -> Result<()> {
    // Parse the manifest and re-encode it in canonical JSON
    let parsed_manifest = serde_json::from_slice(manifest).into_diagnostic()?;
    let custom_section = mrf_manifest::encode(&parsed_manifest).into_diagnostic()?;

    let mut file = File::options()
        .append(true)
        .open(module_path)
        .into_diagnostic()?;
    file.write_all(&custom_section).into_diagnostic()?;

    Ok(())
}

fn main() -> Result<()> {
    let args = ToolArgs::parse();
    match args.command {
        ToolSubcommand::AddManifest(args) => {
            let manifest = fs::read(args.manifest_path).into_diagnostic()?;
            fs::copy(&args.module_path, &args.output).into_diagnostic()?;
            write_manifest(&manifest, &args.output)?;
        }
        ToolSubcommand::ReadManifest(args) => {
            let data = fs::read(args.module_path).into_diagnostic()?;
            read_manifest(&data)?;
        }
        ToolSubcommand::RemoveManifest(args) => {
            remove_manifest(&args.module_path, &args.output)?;
        }
        ToolSubcommand::ValidateModule(args) => {
            let data = fs::read(args.module_path).into_diagnostic()?;
            wasmparser::validate(&data).into_diagnostic()?;
        }
    }

    Ok(())
}
