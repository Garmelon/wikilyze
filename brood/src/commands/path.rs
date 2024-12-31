use std::io;

use crate::{
    algo::Dijkstra,
    data::Data,
    graph::NodeIdx,
    util::{self, TitleNormalizer},
};

/// Find the shortest path between two articles.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    start: String,
    goal: String,

    // Search for a path in both directions.
    #[arg(long, short)]
    bidi: bool,
}

fn search_path(data: &Data, start: NodeIdx, goal: NodeIdx) -> Option<(u32, Vec<NodeIdx>)> {
    println!("> Preparing dijkstra");
    let mut dijkstra = Dijkstra::new(&data.graph);
    println!("> Running dijkstra");
    dijkstra.run(
        start,
        |node| node == goal,
        |source, _edge, _target| !data.pages[source.usize()].redirect as u32,
    );

    if dijkstra.cost(goal) == u32::MAX {
        return None;
    }

    println!("> Collecting path");
    let cost = dijkstra.cost(goal);
    let path = dijkstra.path(goal);
    Some((cost, path))
}

fn print_path(data: &Data, start: NodeIdx, goal: NodeIdx, path: Option<(u32, Vec<NodeIdx>)>) {
    let start = &data.pages[start.usize()].title;
    let goal = &data.pages[goal.usize()].title;

    let Some((cost, path)) = path else {
        println!("No path found from {start} to {goal}");
        return;
    };

    println!("Path found (cost {cost}, length {}):", path.len());

    for page in path {
        println!("{}", util::fmt_page(&data.pages[page.usize()]));
    }
}

impl Cmd {
    pub fn run(self, data: Data) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        println!(">> Resolve articles");
        let start = util::resolve_title(&normalizer, &data, &self.start);
        let goal = util::resolve_title(&normalizer, &data, &self.goal);
        println!("Start: {}", data.pages[start.usize()].title);
        println!("Goal:  {}", data.pages[goal.usize()].title);

        if self.bidi {
            println!(">> Find path forward");
            let forward = search_path(&data, start, goal);
            println!(">> Find path backward");
            let backward = search_path(&data, goal, start);

            println!();
            print_path(&data, start, goal, forward);
            println!();
            print_path(&data, goal, start, backward);
        } else {
            println!(">> Find path");
            let path = search_path(&data, start, goal);

            println!();
            print_path(&data, start, goal, path);
        }

        Ok(())
    }
}
