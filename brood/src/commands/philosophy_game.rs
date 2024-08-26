use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
};

use crate::{
    data::{
        adjacency_list::AdjacencyList,
        info::{LinkInfo, PageInfo},
        store,
    },
    PhilosophyGameCmd,
};

struct PageMap(Vec<u32>);

impl PageMap {
    fn new(len: usize) -> Self {
        Self(vec![u32::MAX; len])
    }

    fn get(&self, page_idx: u32) -> u32 {
        self.0[page_idx as usize]
    }

    fn set(&mut self, page_idx: u32, to: u32) {
        self.0[page_idx as usize] = to;
    }
}

fn first_viable_link(data: &AdjacencyList<PageInfo, LinkInfo>, page_idx: u32) -> Option<u32> {
    for link_idx in data.link_range(page_idx) {
        let link = data.link(link_idx);
        if !link.data.in_parens() && !link.data.in_structure() {
            return Some(link.to);
        }
    }
    None
}

fn find_forward_edges(data: &AdjacencyList<PageInfo, LinkInfo>) -> PageMap {
    let mut result = PageMap::new(data.pages.len());
    for (page_idx, _) in data.pages() {
        if let Some(first_link) = first_viable_link(data, page_idx) {
            result.set(page_idx, first_link);
        }
    }
    result
}

fn find_clusters(data: &AdjacencyList<PageInfo, LinkInfo>, forward: &PageMap) -> PageMap {
    let mut cluster = PageMap::new(data.pages.len());
    for (page_idx, _) in data.pages() {
        let mut current = page_idx;
        let mut visited = HashSet::new();
        let canonical = loop {
            // We've already determined the canonical element for this page.
            if cluster.get(current) != u32::MAX {
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
            if next == u32::MAX {
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

fn print_forward_edges_as_json(
    data: &AdjacencyList<PageInfo, LinkInfo>,
    forward: &PageMap,
) -> io::Result<()> {
    let map = forward
        .0
        .iter()
        .enumerate()
        .map(|(page, first_link)| {
            let page_title = &data.page(page as u32).data.title;
            let first_link_title = if *first_link == u32::MAX {
                None
            } else {
                Some(&data.page(*first_link).data.title)
            };
            (page_title, first_link_title)
        })
        .collect::<HashMap<_, _>>();

    let writer = BufWriter::new(io::stdout());
    serde_json::to_writer_pretty(writer, &map)?;
    Ok(())
}

fn print_canonical_pages_as_json(
    data: &AdjacencyList<PageInfo, LinkInfo>,
    cluster: &PageMap,
) -> io::Result<()> {
    let map = cluster
        .0
        .iter()
        .enumerate()
        .map(|(page, canonical)| {
            (
                &data.page(page as u32).data.title,
                &data.page(*canonical).data.title,
            )
        })
        .collect::<HashMap<_, _>>();

    let writer = BufWriter::new(io::stdout());
    serde_json::to_writer_pretty(writer, &map)?;
    Ok(())
}

pub fn run(datafile: &Path, subcmd: PhilosophyGameCmd) -> io::Result<()> {
    eprintln!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = store::read_adjacency_list(&mut databuf)?;

    eprintln!(">> Forward");
    let forward = find_forward_edges(&data);

    if subcmd == PhilosophyGameCmd::First {
        eprintln!(">> First links");
        print_forward_edges_as_json(&data, &forward)?;
        return Ok(());
    }

    // Determine cluster for each page, represented via canonical page. The
    // canonical page of a cluster is either a dead-end or the loop member with
    // the smallest index.
    eprintln!(">> Find clusters");
    let cluster = find_clusters(&data, &forward);

    if subcmd == PhilosophyGameCmd::Canonical {
        print_canonical_pages_as_json(&data, &cluster)?;
        return Ok(());
    }

    // Measure cluster size
    eprintln!(">> Measure clusters");
    let mut cluster_size = HashMap::<u32, u32>::new();
    for (i, canonical) in cluster.0.iter().enumerate() {
        assert!(*canonical != u32::MAX, "{}", data.page(i as u32).data.title);
        *cluster_size.entry(*canonical).or_default() += 1;
    }
    let mut cluster_by_size = cluster_size.into_iter().collect::<Vec<_>>();
    cluster_by_size.sort_by_key(|(c, s)| (*s, *c));
    cluster_by_size.reverse();

    // Print clusters
    assert!(subcmd == PhilosophyGameCmd::Cluster);
    for (canonical, size) in cluster_by_size {
        if forward.get(canonical) == u32::MAX {
            let title = &data.page(canonical).data.title;
            println!("Cluster (dead-end, {size}): {title}");
            continue;
        }

        println!("Cluster (loop, {size}):");
        let mut current = canonical;
        loop {
            let page = data.page(current);
            let title = &page.data.title;
            if page.data.redirect {
                println!("  v {title}");
            } else {
                println!("  - {title}");
            }
            current = forward.get(current);
            if current == canonical {
                break;
            }
        }
    }

    Ok(())
}
