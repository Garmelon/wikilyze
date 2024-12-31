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
    Show(commands::show::Cmd),
    Path(commands::path::Cmd),
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
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if let Command::Ingest(cmd) = &args.command {
        return cmd.run(&args.datafile);
    }

    println!(">> Import");
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

    match args.command {
        Command::Ingest(_) => unreachable!(),
        Command::Show(cmd) => cmd.run(data),
        Command::Path(cmd) => cmd.run(data),
    }
}
