mod algo;
mod commands;
mod data;
mod graph;
mod util;

use std::{io, path::PathBuf};

use clap::Parser;
use data::Data;

#[derive(Debug, Parser)]
enum Command {
    Ingest(commands::ingest::Cmd),
    Export(commands::export::Cmd),
    Show(commands::show::Cmd),
    Stats(commands::stats::Cmd),
    Path(commands::path::Cmd),
    LongestPath(commands::longest_path::Cmd),
    Pg(commands::pg::Cmd),
}

#[derive(Debug, Parser)]
struct Args {
    datafile: PathBuf,
    #[command(subcommand)]
    command: Command,
    #[arg(long, short = 'P')]
    in_parens: Option<bool>,
    #[arg(long, short = 'S')]
    in_structure: Option<bool>,
    #[arg(long, short = 'R')]
    resolve_redirects: bool,
    #[arg(long, short = 'I')]
    invert_edges: bool,
    #[arg(long, short)]
    check_consistency: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if let Command::Ingest(cmd) = &args.command {
        return cmd.run(&args.datafile);
    }

    println!(">> Import");
    println!("> Reading data");
    let mut data = Data::read_from_file(&args.datafile)?;

    if args.in_parens.is_some() || args.in_structure.is_some() {
        println!("> Filtering edges");
        algo::retain_edges(&mut data, |link| {
            args.in_parens.is_none_or(|b| b == link.in_parens())
                && args.in_structure.is_none_or(|b| b == link.in_structure())
        });
    }

    if args.resolve_redirects {
        println!("> Resolving redirects");
        algo::resolve_redirects(&mut data);
    }

    if args.invert_edges {
        println!("> Inverting edges");
        algo::invert(&mut data);
    }

    if args.check_consistency {
        println!("> Checking consistencey");
        data.check_consistency();
    }

    match args.command {
        Command::Ingest(_) => unreachable!(),
        Command::Export(cmd) => cmd.run(data),
        Command::Show(cmd) => cmd.run(data),
        Command::Stats(cmd) => cmd.run(data),
        Command::Path(cmd) => cmd.run(data),
        Command::LongestPath(cmd) => cmd.run(data),
        Command::Pg(cmd) => cmd.run(data),
    }
}
