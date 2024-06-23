use self::args::{ToolArgs, ToolSubcommand};
use clap::Parser;

mod args;
mod parse_header;
mod util;

fn main() -> miette::Result<()> {
    let args = ToolArgs::parse();
    match args.subcommand {
        ToolSubcommand::ParseHeader(args) => parse_header::do_it(args.header.leak(), args.scheme),
    }
}
