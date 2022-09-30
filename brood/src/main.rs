mod ingest;
mod data;
mod test;

use std::io;

use clap::Parser;

#[derive(Debug, Parser)]
enum Command {
    /// Read sift data on stdin and output brood data on stdout.
    Ingest,
    /// Test various things
    Test,
}

fn main() -> io::Result<()> {
    match Command::parse() {
        Command::Ingest => ingest::ingest(),
        Command::Test => test::test(),
    }
}
