use std::{cmp::Reverse, collections::HashSet, io};

use thousands::Separable;

use crate::{data::Data, graph::NodeIdx, util};

fn find_redirects(data: &Data) -> Vec<(NodeIdx, NodeIdx, usize)> {
    let mut redirects = Vec::<(NodeIdx, NodeIdx, usize)>::new();

    for node in data.graph.nodes() {
        if !data.pages[node.usize()].redirect {
            continue;
        }

        let mut seen = HashSet::new();

        let mut curr = node;
        seen.insert(node);

        while let Some(next) = data.redirect_target(curr) {
            if seen.contains(&next) {
                println!("  Redirect loop: {}", data.pages[node.usize()].title);
                break;
            }

            curr = next;
            seen.insert(next);
        }

        redirects.push((node, curr, seen.len() - 1));
    }

    redirects
}

fn follow_redirect(data: &Data, start: NodeIdx) -> Vec<NodeIdx> {
    let mut seen = HashSet::new();
    let mut nodes = Vec::new();

    let mut curr = start;
    seen.insert(curr);
    nodes.push(curr);

    while let Some(next) = data.redirect_target(curr) {
        if seen.contains(&next) {
            break;
        }

        curr = next;
        seen.insert(curr);
        nodes.push(curr);
    }

    nodes
}

/// Show redirect stats.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    /// Show more detailed info.
    #[arg(long, short)]
    long: bool,
}

impl Cmd {
    pub fn run(self, data: Data) -> io::Result<()> {
        println!(">> Resolve redirects");
        let redirects = find_redirects(&data);

        println!(
            "There is a total of {} redirects.",
            redirects.len().separate_with_underscores()
        );

        let mut long = redirects
            .iter()
            .filter(|(_, _, l)| *l > 1)
            .collect::<Vec<_>>();
        long.sort_by_key(|(_, _, l)| Reverse(l));

        println!(
            "{} redirects take more than one step to reach an article.",
            long.len().separate_with_underscores()
        );

        println!(
            "The longest redirect chain takes {} steps.",
            long.iter().map(|(_, _, l)| l).max().copied().unwrap_or(0),
        );

        println!("Though these redirect chains are usually swiftly fixed by bots.");

        if self.long {
            println!();
            println!("Redirect chains with length > 1:");

            for (start, _, _) in long {
                println!();
                for step in follow_redirect(&data, *start) {
                    println!("{}", util::fmt_page(&data.pages[step.usize()]));
                }
            }
        }

        Ok(())
    }
}
