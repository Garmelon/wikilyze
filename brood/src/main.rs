mod ingest;
mod data;

use std::io;

use clap::Parser;

#[derive(Debug, Parser)]
enum Command {
    /// Read sift data on stdin and output brood data on stdout.
    Ingest,
}

fn main() -> io::Result<()> {
    match Command::parse() {
        Command::Ingest => ingest::ingest(),
    }
}
