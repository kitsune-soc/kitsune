use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct AddManifest {
    /// Path to the manifest
    pub manifest_path: PathBuf,

    /// Path to the WASM module
    pub module_path: PathBuf,

    /// Path to where the modified WASM module should be written
    #[arg(long, short)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ReadManifest {
    /// Path to the WASM module
    pub module_path: PathBuf,
}

#[derive(Args)]
pub struct RemoveManifest {
    /// Path to the WASM module
    pub module_path: PathBuf,

    /// Path to where the modified WASM module should be written
    #[arg(long, short)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ValidateModule {
    /// Path to the WASM module
    pub module_path: PathBuf,
}

#[derive(Subcommand)]
pub enum ManifestSubcommand {
    /// Add a manifest to a WASM component
    ///
    /// Note: We don't validate whether the WASM component already contains a manifest section.
    /// We simply append a new section.
    Add(AddManifest),

    /// Read the manifest from a WASM component
    Read(ReadManifest),

    /// Remove the manifest from a WASM component
    Remove(RemoveManifest),
}

#[derive(Subcommand)]
pub enum ModuleSubcommand {
    /// Validate a WASM module
    Validate(ValidateModule),
}

#[derive(Subcommand)]
pub enum ToolSubcommand {
    /// Manage manifests embedded into modules
    #[clap(subcommand)]
    Manifest(ManifestSubcommand),

    /// Manage WASM MRF modules
    #[clap(subcommand)]
    Module(ModuleSubcommand),
}

#[derive(Parser)]
#[command(about, version)]
pub struct ToolArgs {
    #[clap(subcommand)]
    pub command: ToolSubcommand,
}
