use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::data::{AdjacencyList, Page, PageInfo};
use crate::util;

fn find_index_of_title(pages: &[Page<PageInfo>], title: &str) -> u32 {
    let title = util::normalize_link(title);
    pages
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.data.redirect)
        .find(|(_, p)| util::normalize_link(&p.data.title) == title)
        .map(|(i, _)| i)
        .expect("invalid title") as u32
}

struct DijkstraPageInfo {
    distance: u32,
    prev_page_idx: u32,
}

impl Default for DijkstraPageInfo {
    fn default() -> Self {
        Self {
            distance: u32::MAX,
            prev_page_idx: u32::MAX,
        }
    }
}

fn dijkstra(
    mut data: AdjacencyList<DijkstraPageInfo, ()>,
    from_idx: u32,
    to_idx: u32,
) -> Option<Vec<u32>> {
    todo!()
}

pub fn path(datafile: &Path, from: &str, to: &str) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = AdjacencyList::read(&mut databuf)?;
    let pages = data.pages.clone();
    let data = data
        .change_page_data(&|_| DijkstraPageInfo::default())
        .change_link_data(&|_| ());

    println!(">> Locate from and to");
    let from_idx = find_index_of_title(&pages, from);
    let to_idx = find_index_of_title(&pages, to);

    println!(">> Find path");
    let path = dijkstra(data, from_idx, to_idx);

    if let Some(path) = path {
        println!("Path found:");
        for page_idx in path {
            let page = &pages[page_idx as usize];
            println!(" - {}", page.data.title);
        }
    } else {
        println!("No path found");
    }

    Ok(())
}
