use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::data::{
    adjacency_list::AdjacencyList,
    info::{LinkInfo, PageInfo},
    store,
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

pub fn run(datafile: &Path) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = store::read_adjacency_list(&mut databuf)?;

    // Compute forward and backward edges
    let mut forward = PageMap::new(data.pages.len());
    for (page_idx, _) in data.pages() {
        if let Some(first_link) = first_viable_link(&data, page_idx) {
            forward.set(page_idx, first_link);
        }
    }

    // Determine cluster for each page, represented via canonical page. The
    // canonical page of a cluster is either a dead-end or the loop member with
    // the smallest index.
    println!(">> Cluster");
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

    // Measure cluster size
    let mut cluster_size = HashMap::<u32, u32>::new();
    for (i, canonical) in cluster.0.iter().enumerate() {
        assert!(*canonical != u32::MAX, "{}", data.page(i as u32).data.title);
        *cluster_size.entry(*canonical).or_default() += 1;
    }

    let mut cluster_by_size = cluster_size.into_iter().collect::<Vec<_>>();
    cluster_by_size.sort_by_key(|(c, s)| (*s, *c));

    // Print clusters
    for (canonical, size) in cluster_by_size {
        if forward.get(canonical) == u32::MAX {
            let title = &data.page(canonical).data.title;
            println!("Cluster (dead-end, {size}): {title}");
            continue;
        }

        println!("Cluster (loop, {size}):");
        let mut current = canonical;
        loop {
            let title = &data.page(current).data.title;
            println!("  - {title}");
            current = forward.get(current);
            if current == canonical {
                break;
            }
        }
    }

    Ok(())
}
