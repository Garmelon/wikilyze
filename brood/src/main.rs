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
    /// Read and reexport brood data.
    Reexport { to: PathBuf },
    /// Find a path from one article to another.
    Path {
        from: String,
        to: String,
        /// Flip start and end article.
        #[arg(short, long)]
        flip: bool,
    },
    /// Find the longest shortest path starting at an article.
    LongestShortestPath { from: String },
    /// Print all page titles.
    ListPages,
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
        Command::Ingest => commands::ingest::ingest(&args.datafile),
        Command::Reexport { to } => commands::reexport::reexport(&args.datafile, &to),
        Command::Path { from, to, flip } => {
            if flip {
                commands::path::path(&args.datafile, &to, &from)
            } else {
                commands::path::path(&args.datafile, &from, &to)
            }
        }
        Command::LongestShortestPath { from } => {
            commands::longest_shortest_path::run(&args.datafile, &from)
        }
        Command::ListPages => commands::list_pages::run(&args.datafile),
    }
}
