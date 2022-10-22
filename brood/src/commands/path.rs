use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::data::{AdjacencyList, LinkInfo, Page, PageInfo};
use crate::util;

fn find_index_of_title(pages: &[Page<PageInfo>], title: &str) -> u32 {
    let title = util::normalize_link(title);
    pages
        .iter()
        .enumerate()
        .find(|(_, p)| util::normalize_link(&p.data.title) == title)
        .map(|(i, _)| i)
        .expect("invalid title") as u32
}

fn resolve_redirects(data: &AdjacencyList<PageInfo, LinkInfo>, mut page_idx: u32) -> u32 {
    loop {
        let page = &data.page(page_idx);
        if page.data.redirect {
            page_idx = data.link(page.link_idx).to;
        } else {
            break page_idx;
        }
    }
}

struct DijkstraPageInfo {
    cost: u32,
    prev_page_idx: u32,
    redirect: bool,
}

impl DijkstraPageInfo {
    fn from_page_info(info: PageInfo) -> Self {
        Self {
            cost: u32::MAX,
            prev_page_idx: u32::MAX,
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
            // cost: 1000 + info.start / 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Entry {
    cost: u32,
    page_idx: u32,
}

impl Entry {
    pub fn new(cost: u32, page_idx: u32) -> Self {
        Self { cost, page_idx }
    }
}

// Manual implementation so the queue is a min-heap instead of a max-heap.
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.page_idx.cmp(&other.page_idx))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Closely matches the dijkstra example in [std::collections::binary_heap].
fn dijkstra(
    data: AdjacencyList<PageInfo, LinkInfo>,
    from_idx: u32,
    to_idx: u32,
) -> Option<Vec<u32>> {
    println!("> Prepare state");
    let mut data = data
        .change_page_data(DijkstraPageInfo::from_page_info)
        .change_link_data(DijkstraLinkInfo::from_link_info);
    let mut queue = BinaryHeap::new();
    data.page_mut(from_idx).data.cost = 0;
    queue.push(Entry::new(0, from_idx));

    println!("> Run dijkstra");
    while let Some(Entry { cost, page_idx }) = queue.pop() {
        if page_idx == to_idx {
            // We've found the shortest path to our target
            break;
        }

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
                page_idx: link.to,
            };

            let mut target_page = data.page_mut(link.to);
            if next.cost < target_page.data.cost {
                target_page.data.cost = next.cost;
                target_page.data.prev_page_idx = page_idx;
                queue.push(next);
            }
        }
    }

    println!("> Collect results");
    let mut steps = vec![];
    let mut at_idx = to_idx;
    loop {
        steps.push(at_idx);
        at_idx = data.page(at_idx).data.prev_page_idx;
        if at_idx == u32::MAX {
            break;
        };
    }
    steps.reverse();
    if steps.first() == Some(&from_idx) {
        Some(steps)
    } else {
        None
    }
}

pub fn path(datafile: &Path, from: &str, to: &str) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = AdjacencyList::read(&mut databuf)?;
    let pages = data.pages.clone();

    println!(">> Locate from and to");
    let from_idx = resolve_redirects(&data, find_index_of_title(&pages, from));
    let to_idx = resolve_redirects(&data, find_index_of_title(&pages, to));

    println!(">> Find path");
    let path = dijkstra(data, from_idx, to_idx);

    if let Some(path) = path {
        println!("Path found:");
        for page_idx in path {
            let page = &pages[page_idx as usize];
            if page.data.redirect {
                println!(" v {}", page.data.title);
            } else {
                println!(" - {}", page.data.title);
            }
        }
    } else {
        println!("No path found");
    }

    Ok(())
}
