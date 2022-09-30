use std::collections::hash_map::Entry;
use std::io::{self, BufRead, BufReader};

use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::data::{AdjacencyList, Link, Page, SlimAdjacencyList};

#[derive(Deserialize)]
struct JsonPage {
    ns: u16,
    id: u32,
    title: String,
    redirect: Option<String>,
    #[serde(default)]
    links: Vec<(String, u32, u32)>,
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
    ///
    /// The first entry with id 0 represents a nonexistent link.
    pages: Vec<Page>,
    /// Map from index in [`Self::titles`] to index in [`Self::pages`] (used during the second pass).
    pages_map: FxHashMap<u32, u32>,
    /// List with link info and index into [`Self::titles`].
    links: Vec<Link>,
    /// List with titles.
    titles: Vec<String>,
    /// Map from title to index in [`Self::titles`] (used during decoding).
    titles_map: FxHashMap<String, u32>,
}

impl FirstStage {
    fn new() -> Self {
        let mut result = Self {
            pages: vec![],
            pages_map: FxHashMap::default(),
            links: vec![],
            titles: vec![],
            titles_map: FxHashMap::default(),
        };
        result.push_page(0, 0, "this link does not exist".to_string(), false);
        result
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

    fn push_page(&mut self, ns: u16, id: u32, title: String, redirect: bool) {
        self.pages.push(Page {
            link_idx: self.links.len() as u32,
            ns,
            id,
            title,
            redirect,
        });
    }

    fn insert_page(&mut self, ns: u16, id: u32, title: String, redirect: bool) {
        // We know we haven't seen the page before
        let title_idx = self.insert_title(title.clone());
        let idx = self.pages.len() as u32;
        self.push_page(ns, id, title, redirect);
        self.pages_map.insert(title_idx, idx);
    }

    fn insert_link(&mut self, to: u32, start: u32, end: u32) {
        self.links.push(Link { to, start, end });
    }

    fn import_json_page(&mut self, page: JsonPage) {
        self.insert_page(page.ns, page.id, page.title, page.redirect.is_some());
        if let Some(to) = page.redirect {
            let to = self.insert_title(to);
            self.insert_link(to, 0, 0);
        } else {
            for (to, start, end) in page.links {
                let to = self.insert_title(to);
                self.insert_link(to, start, end);
            }
        }
    }

    fn finalize(&mut self) {
        self.insert_page(
            0,
            0,
            "dummy page at the end of all pages".to_string(),
            false,
        );
    }
}

fn first_stage() -> io::Result<FirstStage> {
    let mut first_stage = FirstStage::new();
    let mut n = 0;

    let stdin = BufReader::new(io::stdin());
    for line in stdin.lines() {
        // let json_page = serde_json::from_str::<JsonPage>(&line?)?;
        let json_page = simd_json::serde::from_str::<JsonPage>(&mut line?).unwrap();
        first_stage.import_json_page(json_page);

        n += 1;
        if n % 100_000 == 0 {
            eprintln!("{n} imported")
        }
    }

    first_stage.finalize();
    Ok(first_stage)
}

fn second_stage(mut fs: FirstStage) -> AdjacencyList {
    let mut n = 0;

    for link in &mut fs.links {
        if let Some(to) = fs.pages_map.get(&link.to) {
            link.to = *to;
        } else {
            link.to = 0;
        }

        n += 1;
        if n % 10_000_000 == 0 {
            eprintln!("{n} links converted");
        }
    }

    AdjacencyList {
        pages: fs.pages,
        links: fs.links,
    }
}

pub fn ingest() -> io::Result<()> {
    eprintln!("FIRST STAGE");
    let first_stage = first_stage()?;
    eprintln!("SECOND STAGE");
    let second_stage = second_stage(first_stage);

    eprintln!("CONSISTENCY CHECK");
    let range = 0..second_stage.pages.len() as u32;
    for link in &second_stage.links {
        if !range.contains(&link.to) {
            eprintln!("Invalid link detected!");
        }
    }

    eprintln!("EXPORT");
    let data = SlimAdjacencyList::from_alist(second_stage);
    ciborium::ser::into_writer(&data, io::stdout()).unwrap();
    // simd_json::to_writer(io::stdout(), &data).unwrap();

    Ok(())
}
