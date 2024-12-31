use std::{
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, BufWriter},
};

use crate::{
    data::Data,
    graph::NodeIdx,
    util::{self, TitleNormalizer},
};

struct PageMap(Vec<NodeIdx>);

impl PageMap {
    fn new(len: usize) -> Self {
        Self(vec![NodeIdx::NONE; len])
    }

    fn get(&self, node: NodeIdx) -> NodeIdx {
        self.0[node.usize()]
    }

    fn set(&mut self, node: NodeIdx, to: NodeIdx) {
        self.0[node.usize()] = to;
    }
}

fn first_viable_link(data: &Data, node: NodeIdx) -> Option<NodeIdx> {
    for edge in data.graph.edge_slice(node) {
        let link = &data.links[edge.usize()];
        if !link.in_parens() && !link.in_structure() {
            return Some(*edge);
        }
    }
    None
}

fn find_forward_edges(data: &Data) -> PageMap {
    let mut result = PageMap::new(data.pages.len());
    for node in data.graph.nodes() {
        if let Some(first_link) = first_viable_link(data, node) {
            result.set(node, first_link);
        }
    }
    result
}

fn find_clusters(data: &Data, forward: &PageMap) -> PageMap {
    let mut cluster = PageMap::new(data.pages.len());
    for node in data.graph.nodes() {
        let mut current = node;
        let mut visited = HashSet::new();
        let canonical = loop {
            // We've already determined the canonical element for this page.
            if cluster.get(current) != NodeIdx::NONE {
                break cluster.get(current);
            }

            // We've hit a loop
            if visited.contains(&current) {
                let mut loop_members = BTreeSet::new();
                while !loop_members.contains(&current) {
                    loop_members.insert(current);
                    current = forward.get(current);
                }
                break loop_members.pop_first().unwrap();
            }

            visited.insert(current);

            let next = forward.get(current);
            if next == NodeIdx::NONE {
                // We've hit a dead-end
                break current;
            }

            current = next;
        };

        for i in visited {
            cluster.set(i, canonical);
        }
    }

    cluster
}

enum Cluster {
    DeadEnd(NodeIdx),
    Loop(Vec<NodeIdx>),
}

fn resolve_clusters(forward: &PageMap, cluster: &PageMap) -> HashMap<NodeIdx, Cluster> {
    let mut result = HashMap::new();
    for canonical in cluster.0.iter().copied().collect::<HashSet<_>>() {
        if forward.get(canonical) == NodeIdx::NONE {
            result.insert(canonical, Cluster::DeadEnd(canonical));
            continue;
        }

        let mut members = vec![];
        let mut current = canonical;
        loop {
            members.push(current);
            current = forward.get(current);
            if current == canonical {
                break;
            }
        }
        result.insert(canonical, Cluster::Loop(members));
    }

    result
}

fn print_forward_edges_as_json(data: &Data, forward: &PageMap) -> io::Result<()> {
    let map = forward
        .0
        .iter()
        .enumerate()
        .map(|(node, first_link)| {
            let page_title = &data.pages[node].title;
            let first_link_title = if *first_link == NodeIdx::NONE {
                None
            } else {
                Some(&data.pages[first_link.usize()].title)
            };
            (page_title, first_link_title)
        })
        .collect::<HashMap<_, _>>();

    let writer = BufWriter::new(io::stdout());
    serde_json::to_writer_pretty(writer, &map)?;
    Ok(())
}

fn print_trace(normalizer: &TitleNormalizer, data: &Data, forward: &PageMap, start: &str) {
    let start_idx = util::resolve_title(normalizer, data, start);

    let mut current = start_idx;
    let mut visited = HashSet::new();
    loop {
        let page = &data.pages[current.usize()];
        let title = &page.title;
        if page.redirect {
            println!("  v {title}");
        } else {
            println!("  - {title}");
        }

        visited.insert(current);

        let next = forward.get(current);

        if next == NodeIdx::NONE {
            println!("> dead-end reached");
            return;
        }

        if visited.contains(&next) {
            let page = &data.pages[next.usize()];
            let title = &page.title;
            println!("> loop detected ({title})");
            return;
        }

        current = next;
    }
}

fn print_canonical_pages_as_json(data: &Data, cluster: &PageMap) -> io::Result<()> {
    let map = cluster
        .0
        .iter()
        .enumerate()
        .map(|(page, canonical)| {
            (
                &data.pages[page].title,
                &data.pages[canonical.usize()].title,
            )
        })
        .collect::<HashMap<_, _>>();

    let writer = BufWriter::new(io::stdout());
    serde_json::to_writer_pretty(writer, &map)?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq, clap::Parser)]
enum Command {
    First,
    Trace { start: String },
    Canonical,
    Cluster,
}

/// Show interesting stats.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    #[command(subcommand)]
    command: Command,
}

impl Cmd {
    pub fn run(self, data: Data) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        eprintln!(">> Forward");
        let forward = find_forward_edges(&data);

        match self.command {
            Command::First => {
                eprintln!(">> First links");
                print_forward_edges_as_json(&data, &forward)?;
                return Ok(());
            }
            Command::Trace { start } => {
                eprintln!(">> Tracing");
                print_trace(&normalizer, &data, &forward, &start);
                return Ok(());
            }
            _ => {}
        }

        // Determine cluster for each page, represented via canonical page. The
        // canonical page of a cluster is either a dead-end or the loop member with
        // the smallest index.
        eprintln!(">> Find clusters");
        let cluster = find_clusters(&data, &forward);

        if self.command == Command::Canonical {
            print_canonical_pages_as_json(&data, &cluster)?;
            return Ok(());
        }

        // Measure cluster size
        eprintln!(">> Measure clusters");
        let mut cluster_size = HashMap::<NodeIdx, u32>::new();
        for (i, canonical) in cluster.0.iter().enumerate() {
            assert!(*canonical != NodeIdx::NONE, "{}", data.pages[i].title);
            *cluster_size.entry(*canonical).or_default() += 1;
        }
        let mut cluster_by_size = cluster_size.into_iter().collect::<Vec<_>>();
        cluster_by_size.sort_by_key(|(c, s)| (*s, *c));
        cluster_by_size.reverse();

        // Print clusters
        assert!(self.command == Command::Cluster);
        let resolved = resolve_clusters(&forward, &cluster);
        for (canonical, size) in cluster_by_size {
            match resolved.get(&canonical).unwrap() {
                Cluster::DeadEnd(page) => {
                    let title = &data.pages[page.usize()].title;
                    println!("Cluster (dead-end, {size}): {title}");
                }
                Cluster::Loop(pages) => {
                    println!("Cluster ({}-loop, {size}):", pages.len());
                    for page in pages {
                        let page = &data.pages[page.usize()];
                        let title = &page.title;
                        if page.redirect {
                            println!("  v {title}");
                        } else {
                            println!("  - {title}");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
