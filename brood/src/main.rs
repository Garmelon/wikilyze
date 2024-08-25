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
    Reexport {
        to: PathBuf,
        #[arg(long, short = 'P')]
        in_parens: Option<bool>,
        #[arg(long, short = 'S')]
        in_structure: Option<bool>,
    },
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
    /// Analyze articles using "Philosophy Game" rules.
    PhilosophyGame,
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
        Command::Reexport {
            to,
            in_parens,
            in_structure,
        } => commands::reexport::reexport(&args.datafile, &to, in_parens, in_structure),
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
        Command::PhilosophyGame => commands::philosophy_game::run(&args.datafile),
        Command::ListPages => commands::list_pages::run(&args.datafile),
    }
}
