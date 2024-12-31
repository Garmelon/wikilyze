use std::{io, path::Path};

use crate::{
    algo::Dijkstra,
    data::Data,
    util::{self, TitleNormalizer},
};

/// Find the shortest path between two articles.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    start: String,
    goal: String,
}

impl Cmd {
    pub fn run(self, data: &Path) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        println!(">> Import");
        let data = Data::read_from_file(data)?;

        println!(">> Resolve articles");
        let start = util::resolve_title(&normalizer, &data, &self.start);
        let goal = util::resolve_title(&normalizer, &data, &self.goal);
        println!("Start: {}", data.pages[start.usize()].title);
        println!("Goal:  {}", data.pages[goal.usize()].title);

        println!(">> Find path");
        println!("> Preparing dijkstra");
        let mut dijkstra = Dijkstra::new(&data.graph);
        println!("> Running dijkstra");
        dijkstra.run(
            start,
            |node| node == goal,
            |source, _edge, _target| !data.pages[source.usize()].redirect as u32,
        );

        if dijkstra.cost(goal) == u32::MAX {
            println!("No path found");
            return Ok(());
        }

        println!("> Collecting path");
        let path = dijkstra.path(goal);
        let cost = dijkstra.cost(goal);

        println!();
        println!("Path found (cost {cost}, length {}):", path.len());
        for page in path {
            println!("{}", util::fmt_page(&data.pages[page.usize()]));
        }

        Ok(())
    }
}
