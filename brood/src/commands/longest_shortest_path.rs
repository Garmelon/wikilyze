use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::data::adjacency_list::{AdjacencyList, PageIdx};
use crate::data::info::{LinkInfo, PageInfo};
use crate::data::store;
use crate::util;

struct DijkstraPageInfo {
    cost: u32,
    prev: PageIdx,
    redirect: bool,
}

impl DijkstraPageInfo {
    fn from_page_info(info: PageInfo) -> Self {
        Self {
            cost: u32::MAX,
            prev: PageIdx::MAX,
            redirect: info.redirect,
        }
    }
}

struct DijkstraLinkInfo {
    cost: u32,
}

impl DijkstraLinkInfo {
    fn from_link_info(info: LinkInfo) -> Self {
        Self {
            cost: 1,
            // cost: 1000 + info.start,
            // cost: 10000 + info.start,
            // cost: 1000 + info.start / 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Entry {
    cost: u32,
    idx: PageIdx,
}

impl Entry {
    pub fn new(cost: u32, idx: PageIdx) -> Self {
        Self { cost, idx }
    }
}

// Manual implementation so the queue is a min-heap instead of a max-heap.
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.idx.cmp(&other.idx))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Closely matches the dijkstra example in [std::collections::binary_heap].
fn full_dijkstra(
    data: AdjacencyList<PageInfo, LinkInfo>,
    from_idx: PageIdx,
) -> AdjacencyList<DijkstraPageInfo, DijkstraLinkInfo> {
    println!("> Prepare state");
    let mut data = data
        .change_page_data(DijkstraPageInfo::from_page_info)
        .change_link_data(DijkstraLinkInfo::from_link_info);
    let mut queue = BinaryHeap::new();
    data.page_mut(from_idx).data.cost = 0;
    queue.push(Entry::new(0, from_idx));

    println!("> Run dijkstra");
    while let Some(Entry {
        cost,
        idx: page_idx,
    }) = queue.pop()
    {
        let page = data.page(page_idx);
        if cost > page.data.cost {
            // This queue entry is outdated
            continue;
        }

        let redirect = page.data.redirect;
        for link_idx in data.link_range(page_idx) {
            let link = data.link(link_idx);

            let next = Entry {
                cost: cost + if redirect { 0 } else { link.data.cost },
                idx: link.to,
            };

            let target_page = data.page_mut(link.to);
            if next.cost < target_page.data.cost {
                target_page.data.cost = next.cost;
                target_page.data.prev = page_idx;
                queue.push(next);
            }
        }
    }

    data
}

fn find_longest_shortest_path(
    data: AdjacencyList<DijkstraPageInfo, DijkstraLinkInfo>,
    from: PageIdx,
) -> Option<Vec<PageIdx>> {
    let to = PageIdx(
        data.pages
            .iter()
            .enumerate()
            .filter(|(_, p)| p.data.cost != u32::MAX)
            .max_by_key(|(_, p)| p.data.cost)?
            .0 as u32,
    );

    let mut steps = vec![];
    let mut at = to;
    loop {
        steps.push(at);
        at = data.page(at).data.prev;
        if at == PageIdx::MAX {
            break;
        };
    }
    steps.reverse();
    if steps.first() == Some(&from) {
        Some(steps)
    } else {
        None
    }
}

pub fn run(datafile: &Path, from: &str) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = store::read_adjacency_list(&mut databuf)?;
    let pages = data.pages.clone();

    println!(">> Locate from and to");
    let from_idx = util::resolve_redirects(&data, util::find_index_of_title(&pages, from));
    println!("From: {:?}", data.page(from_idx).data.title);

    println!(">> Find all shortest paths");
    let data = full_dijkstra(data, from_idx);

    println!(">> Find longest shortest path");
    let path = find_longest_shortest_path(data, from_idx);

    if let Some(path) = path {
        println!("Path found:");
        for page_idx in path {
            let page = &pages[page_idx.0 as usize];
            if page.data.redirect {
                println!(" v {:?}", page.data.title);
            } else {
                println!(" - {:?}", page.data.title);
            }
        }
    } else {
        println!("No path found");
    }

    Ok(())
}
