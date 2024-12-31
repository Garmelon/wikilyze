use std::io;

use crate::{
    algo::Dijkstra,
    data::Data,
    graph::NodeIdx,
    util::{self, TitleNormalizer},
};

/// Find the article with the longest shortest path away from the starting
/// article.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    start: String,
    #[arg(long, short, default_value_t = 1)]
    top: usize,
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

        println!(">> Resolve article");
        let start = util::resolve_title(&normalizer, &data, &self.start);
        println!("Start: {}", data.pages[start.usize()].title);

        println!(">> Search paths");
        println!("> Preparing dijkstra");
        let mut dijkstra = Dijkstra::new(&data.graph);
        println!("> Running dijkstra");
        dijkstra.run(
            start,
            |_| false,
            |source, _edge, _target| !data.pages[source.usize()].redirect as u32,
        );

        println!(">> Find longest paths");
        let mut costs = data
            .graph
            .nodes()
            .map(|n| (dijkstra.cost(n), n))
            .filter(|(c, _)| *c < u32::MAX) // Only reachable nodes please
            .collect::<Vec<_>>();
        costs.sort_unstable();

        for (cost, goal) in costs.iter().rev().take(self.top) {
            let path = dijkstra.path(*goal);
            println!();
            print_path(&data, start, *goal, Some((*cost, path)));
        }

        Ok(())
    }
}
