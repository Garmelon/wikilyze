use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{self, BufRead, BufReader, Seek},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thousands::Separable;

use crate::{
    data::{self, Link, Page},
    graph::{Graph, NodeIdx},
    util::{Counter, TitleNormalizer},
};

#[derive(Deserialize)]
struct JsonPage {
    id: u32,
    title: String,
    length: u32,
    links: Vec<(String, u32, u32, u8)>,
    redirect: Option<String>,
}

fn read_titles(r: &mut BufReader<File>) -> io::Result<Vec<String>> {
    let mut counter = Counter::new();
    let mut titles = vec![];

    for line in r.lines() {
        counter.tick();
        let page = serde_json::from_str::<JsonPage>(&line?).unwrap();
        titles.push(page.title);
    }

    counter.done();
    Ok(titles)
}

fn compute_title_lookup(normalizer: &TitleNormalizer, titles: &[String]) -> HashMap<String, u32> {
    let mut counter = Counter::new();
    let mut title_lookup = HashMap::new();

    for (i, title) in titles.iter().enumerate() {
        counter.tick();
        match title_lookup.entry(normalizer.normalize(title)) {
            Entry::Occupied(mut entry) => {
                let prev_i = *entry.get();
                let prev = &titles[prev_i as usize];
                if prev == title {
                    println!("  {title:?} ({prev_i}) occurs again at {i}");
                    // Prefer later occurrences of articles over earlier ones under
                    // the assumption that their contents are "fresher".
                    entry.insert(i as u32);
                } else {
                    println!(
                        "  {prev:?} ({prev_i}) and {title:?} ({i}) both normalize to {:?}",
                        normalizer.normalize(title)
                    );
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(i as u32);
            }
        }
    }

    counter.done();
    title_lookup
}

fn read_page_data(
    normalizer: &TitleNormalizer,
    title_lookup: &HashMap<String, u32>,
    r: &mut BufReader<File>,
) -> io::Result<(Vec<Page>, Vec<Link>, Graph)> {
    let mut counter = Counter::new();
    let mut pages = vec![];
    let mut links = vec![];
    let mut graph = Graph::new();

    for (i, line) in r.lines().enumerate() {
        counter.tick();
        let page = serde_json::from_str::<JsonPage>(&line?).unwrap();
        let normalized = normalizer.normalize(&page.title);

        let expected_i = title_lookup[&normalized];
        if i as u32 != expected_i {
            // Articles may occur multiple times, and this is not the instance
            // of the article we should keep.
            println!("  Skipping {:?} ({i}) in favor of {expected_i}", page.title);
            continue;
        }

        graph.add_node();
        pages.push(Page {
            id: page.id,
            title: page.title,
            length: page.length,
            redirect: page.redirect.is_some(),
        });

        let mut page_links = page.links;
        if let Some(target) = page.redirect {
            page_links.clear();
            let len = target.len() as u32;
            page_links.push((target, 0, len, 0));
        }

        for (target, start, len, flags) in page_links {
            if let Some(target_i) = title_lookup.get(&normalizer.normalize(&target)) {
                graph.edges.push(NodeIdx(*target_i));
                links.push(Link { start, len, flags });
            }
        }
    }

    counter.done();
    Ok((pages, links, graph))
}

/// Convert sift data to brood data.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    /// The sift data file to ingest.
    data: PathBuf,
}

impl Cmd {
    pub fn run(self, data: &Path) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        println!(">> First pass");
        let mut sift_data = BufReader::new(File::open(&self.data)?);

        println!("> Reading titles");
        let titles = read_titles(&mut sift_data)?;

        println!("> Computing title index lookup table");
        let title_lookup = compute_title_lookup(&normalizer, &titles);
        drop(titles); // Don't hoard memory

        println!(">> Second pass");
        sift_data.seek(io::SeekFrom::Start(0))?;

        println!("> Reading page data");
        let (pages, links, graph) = read_page_data(&normalizer, &title_lookup, &mut sift_data)?;
        drop(title_lookup); // Don't hoard memory
        drop(sift_data); // No longer needed

        println!("> Checking consistency");
        graph.check_consistency();

        println!(">> Export");
        println!("Pages: {}", pages.len().separate_with_underscores());
        println!("Links: {}", links.len().separate_with_underscores());
        data::write_to_file(data, &pages, &links, &graph)?;

        Ok(())
    }
}
