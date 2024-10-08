use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::Path;
use std::u32;

use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::data::adjacency_list::{AdjacencyList, Page};
use crate::data::info::{LinkInfo, PageInfo};
use crate::data::store;
use crate::util;

#[derive(Deserialize)]
struct JsonPage {
    id: u32,
    title: String,
    length: u32,
    links: Vec<(String, u32, u32, u8)>,
    redirect: Option<String>,
}

/*
Importing is a tad complicated because of multiple criteria:

1. The data must be read in a single pass on stdin
2. The process should not consume a lot of memory
   (can't store the decoded json data directly)
3. The process should result in a nice and compact adjacency list format

Because of this, the import is a bit more complex and has two passes.

The first pass imports the data into an adjacency-list-like format, but the
`Link::to` field points to a title in `Titles` instead of a page.

The second pass then resolves the links to page indices and throws away all
links that don't point to any known page.
*/

#[derive(Default)]
struct Titles {
    /// Normalized titles
    titles: Vec<String>,
    /// Map from normalized title to index in [`Self::titles`].
    map: FxHashMap<String, u32>,
}

impl Titles {
    fn insert(&mut self, title: String) -> u32 {
        match self.map.entry(title.clone()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let idx = self.titles.len() as u32;
                self.titles.push(title);
                vacant.insert(idx);
                idx
            }
        }
    }

    fn get(&self, i: u32) -> &str {
        &self.titles[i as usize]
    }
}

fn first_stage() -> io::Result<(AdjacencyList<PageInfo, LinkInfo>, Titles)> {
    let mut titles = Titles::default();
    let mut result = AdjacencyList::default();

    let stdin = BufReader::new(io::stdin());
    for (i, line) in stdin.lines().enumerate() {
        let json_page = serde_json::from_str::<JsonPage>(&line?).unwrap();

        result.push_page(PageInfo {
            id: json_page.id,
            length: json_page.length,
            redirect: json_page.redirect.is_some(),
            title: json_page.title,
        });

        if let Some(to) = json_page.redirect {
            let to = titles.insert(util::normalize_link(&to));
            result.push_link(to, LinkInfo::default());
        } else {
            for (to, start, len, flags) in json_page.links {
                let to = titles.insert(util::normalize_link(&to));
                result.push_link(to, LinkInfo { start, len, flags });
            }
        }

        if (i + 1) % 100_000 == 0 {
            eprintln!("{} pages imported", i + 1)
        }
    }

    eprintln!("Pages: {}", result.pages.len());
    eprintln!("Links: {}", result.links.len());
    eprintln!("Titles: {}", titles.titles.len());
    eprintln!("Title map entries: {}", titles.map.len());

    Ok((result, titles))
}

/// Create map from normalized title to index in pages.
fn initialize_pages_map(pages: &[Page<PageInfo>]) -> FxHashMap<String, u32> {
    let mut result = FxHashMap::default();
    for (i, p) in pages.iter().enumerate() {
        match result.entry(util::normalize_link(&p.data.title)) {
            Entry::Occupied(entry) => {
                eprintln!(
                    "{:?} already exists at index {} as {:?}",
                    p.data.title,
                    entry.get(),
                    util::normalize_link(&p.data.title)
                );
            }
            Entry::Vacant(entry) => {
                entry.insert(i as u32);
            }
        }
    }
    result
}

fn second_stage(
    first_stage: &AdjacencyList<PageInfo, LinkInfo>,
    titles: &Titles,
) -> AdjacencyList<PageInfo, LinkInfo> {
    let pages_map = initialize_pages_map(&first_stage.pages);
    let mut result = AdjacencyList::default();

    for (page_idx, page) in first_stage.pages() {
        result.push_page(page.data.clone());

        for (_, link) in first_stage.links(page_idx) {
            let title = util::normalize_link(titles.get(link.to));
            if let Some(to) = pages_map.get(&title) {
                // The link points to an existing article, we should keep it
                result.push_link(*to, link.data);
            }
        }

        if (page_idx + 1) % 100_000 == 0 {
            eprintln!("{} pages imported", page_idx + 1)
        }
    }

    eprintln!("Pages: {}", result.pages.len());
    eprintln!("Links: {}", result.links.len());
    eprintln!("Page map entries: {}", pages_map.len());

    result
}

pub fn ingest(datafile: &Path) -> io::Result<()> {
    eprintln!(">> First stage");
    let (first_stage, titles) = first_stage()?;

    eprintln!(">> Second stage");
    let data = second_stage(&first_stage, &titles);

    eprintln!(">> Consistency check");
    data.check_consistency();

    eprintln!(">> Export");
    let mut datafile = BufWriter::new(File::create(datafile)?);
    store::write_adjacency_list(&data, &mut datafile)?;

    Ok(())
}
