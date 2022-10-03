pub mod commands;
mod data;
mod util;

use std::io;
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
enum Command {
    /// Read sift data on stdin and output brood data.
    Ingest,
}

#[derive(Debug, Parser)]
struct Args {
    datafile: PathBuf,
    #[command(subcommand)]
    command: Command,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Ingest => commands::ingest(&args.datafile),
    }
}
