use std::collections::hash_map::Entry;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::data::{AdjacencyList, Link, Page};
use crate::util;

#[derive(Deserialize)]
struct JsonPage {
    id: u32,
    title: String,
    length: u32,
    links: Vec<(String, u32, u32)>,
    redirect: Option<String>,
}

/*
Importing is a tad complicated because of multiple criteria:

1. The data must be read in a single pass on stdin
2. The process should not consume a lot of memory
   (can't store the decoded json data directly)
3. The process should result in a nice and compact adjacency list format

Because of this, the import is a bit more complex and has two passes.

The first pass imports the data into an adjacency-list-like format:
- `pages`: List with page info and index in `links`
- `links`: List with link info and index in `titles`
- `titles`: List with titles
- `titles_map`: Map from title to index in `titles` (used during decoding)

The second pass then takes 1 and 3 and changes the indices in 3 to point to the
entries in 1 using 2 and 4. After this, 2, 4 and 5 can be discarded and 1 and 3
form a proper adjacency list.
*/

struct FirstStage {
    /// List with page info and index into [`Self::links`].
    pages: Vec<Page>,
    /// List with link info and index into [`Self::titles`].
    links: Vec<Link>,
    /// List with titles.
    titles: Vec<String>,
    /// Map from normalized title to index in [`Self::titles`].
    titles_map: FxHashMap<String, u32>,
}

impl FirstStage {
    fn new() -> Self {
        Self {
            pages: vec![],
            links: vec![],
            titles: vec![],
            titles_map: FxHashMap::default(),
        }
    }

    fn insert_title(&mut self, title: String) -> u32 {
        match self.titles_map.entry(title.clone()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let idx = self.titles.len() as u32;
                self.titles.push(title);
                vacant.insert(idx);
                idx
            }
        }
    }

    fn insert_page(&mut self, id: u32, title: String, redirect: bool) {
        let link_idx = self.pages.len() as u32;
        self.pages.push(Page {
            link_idx,
            id,
            title,
            redirect,
        });
    }

    fn insert_link(&mut self, to: u32, start: u32, end: u32) {
        self.links.push(Link { to, start, end });
    }

    fn import_json_page(&mut self, page: JsonPage) {
        self.insert_page(page.id, page.title, page.redirect.is_some());
        for (to, start, end) in page.links {
            let to = self.insert_title(util::normalize_link(&to));
            self.insert_link(to, start, end);
        }
    }

    fn finalize(&mut self) {
        self.insert_page(0, "dummy page at the end of all pages".to_string(), false);
    }

    fn from_stdin() -> io::Result<Self> {
        let mut result = Self::new();

        let stdin = BufReader::new(io::stdin());
        for (i, line) in stdin.lines().enumerate() {
            // let json_page = serde_json::from_str::<JsonPage>(&line?)?;
            let json_page = simd_json::serde::from_str::<JsonPage>(&mut line?).unwrap();
            result.import_json_page(json_page);

            if (i + 1) % 100_000 == 0 {
                eprintln!("{} pages imported", i + 1)
            }
        }

        result.finalize();
        Ok(result)
    }
}

struct SecondStage {
    /// List with page info and index into [`Self::links`].
    pages: Vec<Page>,
    /// List with link info and index into [`Self::pages`].
    links: Vec<Link>,
    /// Map from normalized title to index in [`Self::pages`].
    pages_map: FxHashMap<String, u32>,
}

impl SecondStage {
    fn new() -> Self {
        Self {
            pages: vec![],
            links: vec![],
            pages_map: FxHashMap::default(),
        }
    }

    fn initialize_pages_map(&mut self, pages: &[Page]) {
        for (idx, page) in pages.iter().enumerate() {
            let title = util::normalize_link(&page.title);
            self.pages_map.insert(title, idx as u32);
        }
    }

    fn insert_page(&mut self, page: &Page) {
        let mut page = page.clone();
        page.link_idx = self.pages.len() as u32;
        self.pages.push(page);
    }

    fn insert_link(&mut self, mut link: Link, titles: &[String]) {
        let title = &titles[link.to as usize];
        if let Some(page_idx) = self.pages_map.get(title) {
            link.to = *page_idx;
            self.links.push(link);
        }
    }

    fn finalize(&mut self, pages: &[Page]) {
        self.insert_page(pages.last().unwrap());
    }

    fn from_first_stage(first_stage: FirstStage) -> Self {
        drop(first_stage.titles_map);

        let mut result = Self::new();

        eprintln!("> Initializing pages map");
        result.initialize_pages_map(&first_stage.pages);

        eprintln!("> Rearranging links");
        for page_idx in 0..first_stage.pages.len() - 1 {
            let page = &first_stage.pages[page_idx];
            result.insert_page(page);

            let next_link_idx = first_stage.pages[page_idx + 1].link_idx;
            for link_idx in page.link_idx..next_link_idx {
                let link = first_stage.links[link_idx as usize];
                result.insert_link(link, &first_stage.titles);
            }

            if (page_idx + 1) % 100_000 == 0 {
                eprintln!("{} pages updated", page_idx + 1);
            }
        }

        result.finalize(&first_stage.pages);
        result
    }

    fn into_adjacency_list(self) -> AdjacencyList {
        AdjacencyList {
            pages: self.pages,
            links: self.links,
        }
    }
}

pub fn ingest(datafile: &Path) -> io::Result<()> {
    eprintln!(">> First stage");
    let first_stage = FirstStage::from_stdin()?;

    eprintln!(">> Second stage");
    let second_stage = SecondStage::from_first_stage(first_stage);

    let data = second_stage.into_adjacency_list();

    eprintln!(">> Consistency check");
    let range = 0..data.pages.len() as u32;
    for link in &data.links {
        if !range.contains(&link.to) {
            eprintln!("Invalid link detected!");
        }
    }

    // eprintln!("EXPORT");
    // let data = SlimAdjacencyList::from_alist(second_stage);
    // ciborium::ser::into_writer(&data, io::stdout()).unwrap();
    // simd_json::to_writer(io::stdout(), &data).unwrap();

    Ok(())
}
