use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Default, PartialEq, ValueEnum)]
pub enum SignatureScheme {
    #[default]
    Cavage,
}

#[derive(Args)]
pub struct ParseHeaderArgs {
    /// The header to parse
    pub header: String,

    /// The scheme this header uses
    #[arg(default_value_t, long, short, value_enum)]
    pub scheme: SignatureScheme,
}

#[derive(Subcommand)]
pub enum ToolSubcommand {
    /// Parse the HTTP Signature header and report any format errors
    ParseHeader(ParseHeaderArgs),
}

#[derive(Parser)]
#[command(about, version)]
pub struct ToolArgs {
    #[clap(subcommand)]
    pub subcommand: ToolSubcommand,
}
