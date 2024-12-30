mod algo;
pub mod commands;
mod data;
mod graph;
mod util;

use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use data::store;

#[derive(Debug, PartialEq, Eq, Parser)]
pub enum PhilosophyGameCmd {
    First,
    Canonical,
    Cluster,
    Trace { start: String },
}

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
    LongestShortestPath {
        from: String,
    },
    /// Analyze articles using "Philosophy Game" rules.
    PhilosophyGame {
        #[command(subcommand)]
        subcmd: PhilosophyGameCmd,
    },
    /// Print all page titles.
    ListPages,
    /// Print all links.
    ListLinks {
        /// The page to inspect.
        page: String,
    },
    Test,
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
        Command::PhilosophyGame { subcmd } => {
            commands::philosophy_game::run(&args.datafile, subcmd)
        }
        Command::ListPages => commands::list_pages::run(&args.datafile),
        Command::ListLinks { page } => commands::list_links::run(&args.datafile, &page),
        Command::Test => test(&args.datafile),
    }
}

fn test(datafile: &Path) -> io::Result<()> {
    let a = Instant::now();
    // println!(">> Import adjacency list");
    // let mut databuf = BufReader::new(File::open(datafile)?);
    // let adjlist = store::read_adjacency_list(&mut databuf)?;
    println!(">> Import graph");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let (pages, links, graph) = store::read_graph(&mut databuf)?;
    let b = Instant::now();

    println!("{:?}", b.duration_since(a));

    Ok(())
}
