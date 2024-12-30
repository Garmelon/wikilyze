mod algo;
mod commands;
mod data;
mod graph;
mod util;

use std::{io, path::PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
enum Command {
    Ingest(commands::ingest::Cmd),
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
        Command::Ingest(cmd) => cmd.run(&args.datafile),
    }
}
