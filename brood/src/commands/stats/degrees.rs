use std::{cmp::Reverse, io};

use crate::{
    algo,
    data::{Data, Page},
    util,
};

/// Show stats on article in- and out-degrees.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    #[arg(long, short, default_value_t = 5)]
    top: usize,
}

impl Cmd {
    pub fn run(self, mut data: Data) -> io::Result<()> {
        println!(">> Outdegree");
        println!("> Counting links");
        let mut outdegree = vec![usize::MAX; data.pages.len()];
        for node in data.graph.nodes() {
            outdegree[node.usize()] = data.graph.edge_range(node).len();
        }

        println!(">> Indegree");
        println!("> Inverting edges");
        algo::invert(&mut data);
        let mut indegree = vec![usize::MAX; data.pages.len()];
        println!("> Counting links");
        for node in data.graph.nodes() {
            indegree[node.usize()] = data.graph.edge_range(node).len();
        }

        let mut by_degrees = data
            .pages
            .iter()
            .zip(outdegree)
            .zip(indegree)
            .map(|((p, od), id)| (p, od, id))
            .collect::<Vec<_>>();

        println!();
        println!("Most outlinks");
        println!("¯¯¯¯¯¯¯¯¯¯¯¯¯");

        by_degrees.sort_by_key(|(_, od, _)| Reverse(*od));
        self.print_links(&by_degrees);

        println!();
        println!("Most inlinks");
        println!("¯¯¯¯¯¯¯¯¯¯¯¯");

        by_degrees.sort_by_key(|(_, _, id)| Reverse(*id));
        self.print_links(&by_degrees);

        by_degrees.retain(|(_, od, id)| *od > 0 && *id > 0);

        println!();
        println!("Most outlinks per non-zero inlink");
        println!("¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯");

        by_degrees.sort_by(|(_, od1, id1), (_, od2, id2)| {
            let r1 = *od1 as f32 / *id1 as f32;
            let r2 = *od2 as f32 / *id2 as f32;
            r2.total_cmp(&r1) // Reverse order so max values are at beginnibg
        });
        self.print_links(&by_degrees);

        println!();
        println!("Most inlinks per non-zero outlink");
        println!("¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯");

        by_degrees.reverse();
        self.print_links(&by_degrees);

        Ok(())
    }

    fn print_links(&self, by_degrees: &Vec<(&Page, usize, usize)>) {
        for (page, od, id) in by_degrees.iter().take(self.top) {
            println!("{} ({od} out, {id} in)", util::fmt_page(page));
        }
    }
}
